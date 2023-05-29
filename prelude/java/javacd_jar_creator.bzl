# Copyright (c) Meta Platforms, Inc. and affiliates.
#
# This source code is licensed under both the MIT license found in the
# LICENSE-MIT file in the root directory of this source tree and the Apache
# License, Version 2.0 found in the LICENSE-APACHE file in the root directory
# of this source tree.

load(
    "@prelude//java:java_providers.bzl",
    "make_compile_outputs",
)
load("@prelude//java:java_resources.bzl", "get_resources_map")
load(
    "@prelude//java:java_toolchain.bzl",
    "AbiGenerationMode",  # @unused Used as a type
    "DepFiles",
)
load(
    "@prelude//jvm:cd_jar_creator_util.bzl",
    "OutputPaths",
    "TargetType",
    "add_java_7_8_bootclasspath",
    "add_output_paths_to_cmd_args",
    "base_qualified_name",
    "declare_prefixed_output",
    "define_output_paths",
    "encode_base_jar_command",
    "encode_jar_params",
    "generate_abi_jars",
    "get_abi_generation_mode",
    "get_compiling_deps_tset",
    "prepare_final_jar",
    "setup_dep_files",
)
load("@prelude//utils:utils.bzl", "expect")

base_command_params = struct(
    withDownwardApi = True,
    spoolMode = "DIRECT_TO_JAR",
)

def create_jar_artifact_javacd(
        actions: "actions",
        actions_identifier: [str.type, None],
        abi_generation_mode: [AbiGenerationMode.type, None],
        java_toolchain: "JavaToolchainInfo",
        label,
        output: ["artifact", None],
        javac_tool: ["", None],
        srcs: ["artifact"],
        remove_classes: [str.type],
        resources: ["artifact"],
        resources_root: [str.type, None],
        manifest_file: ["artifact", None],
        ap_params: ["AnnotationProcessorParams"],
        plugin_params: ["PluginParams", None],
        source_level: int.type,
        target_level: int.type,
        deps: ["dependency"],
        required_for_source_only_abi: bool.type,
        source_only_abi_deps: ["dependency"],
        extra_arguments: "cmd_args",
        additional_classpath_entries: ["artifact"],
        additional_compiled_srcs: ["artifact", None],
        bootclasspath_entries: ["artifact"],
        is_building_android_binary: bool.type,
        is_creating_subtarget: bool.type = False) -> "JavaCompileOutputs":
    if javac_tool != None:
        # TODO(cjhopman): We can probably handle this better. I think we should be able to just use the non-javacd path.
        fail("cannot set explicit javac on library when using javacd")

    resources_map = get_resources_map(java_toolchain, label.package, resources, resources_root)

    # TODO(cjhopman): Handle manifest file.
    _ = manifest_file

    bootclasspath_entries = add_java_7_8_bootclasspath(target_level, bootclasspath_entries, java_toolchain)
    abi_generation_mode = get_abi_generation_mode(abi_generation_mode, java_toolchain, srcs, ap_params)

    should_create_class_abi = not is_creating_subtarget and (abi_generation_mode == AbiGenerationMode("class") or not is_building_android_binary)
    if should_create_class_abi:
        class_abi_jar = declare_prefixed_output(actions, actions_identifier, "class-abi.jar")
        class_abi_output_dir = declare_prefixed_output(actions, actions_identifier, "class_abi_dir", dir = True)
    else:
        class_abi_jar = None
        class_abi_output_dir = None

    output_paths = define_output_paths(actions, actions_identifier, label)
    path_to_class_hashes_out = declare_prefixed_output(actions, actions_identifier, "classes.txt")

    compiling_deps_tset = get_compiling_deps_tset(actions, deps, additional_classpath_entries)

    # external javac does not support used classes
    track_class_usage = javac_tool == None

    def encode_library_command(
            output_paths: OutputPaths.type,
            path_to_class_hashes: "artifact",
            classpath_jars_tag: "artifact_tag") -> struct.type:
        target_type = TargetType("library")

        base_jar_command = encode_base_jar_command(
            javac_tool,
            target_type,
            output_paths,
            remove_classes,
            label,
            compiling_deps_tset,
            classpath_jars_tag,
            bootclasspath_entries,
            source_level,
            target_level,
            abi_generation_mode,
            srcs,
            resources_map,
            ap_params,
            plugin_params,
            extra_arguments,
            source_only_abi_compiling_deps = [],
            track_class_usage = track_class_usage,
        )

        return struct(
            baseCommandParams = base_command_params,
            libraryJarCommand = struct(
                baseJarCommand = base_jar_command,
                libraryJarBaseCommand = struct(
                    pathToClasses = output_paths.jar.as_output(),
                    rootOutput = output_paths.jar_parent.as_output(),
                    pathToClassHashes = path_to_class_hashes.as_output(),
                    annotationsPath = output_paths.annotations.as_output(),
                ),
            ),
        )

    def encode_abi_command(
            output_paths: OutputPaths.type,
            target_type: TargetType.type,
            classpath_jars_tag: "artifact_tag",
            source_only_abi_compiling_deps: ["JavaClasspathEntry"] = []) -> struct.type:
        base_jar_command = encode_base_jar_command(
            javac_tool,
            target_type,
            output_paths,
            remove_classes,
            label,
            compiling_deps_tset,
            classpath_jars_tag,
            bootclasspath_entries,
            source_level,
            target_level,
            abi_generation_mode,
            srcs,
            resources_map,
            ap_params,
            plugin_params,
            extra_arguments,
            source_only_abi_compiling_deps = source_only_abi_compiling_deps,
            track_class_usage = track_class_usage,
        )
        abi_params = encode_jar_params(remove_classes, output_paths)

        abi_command = struct(
            baseJarCommand = base_jar_command,
            abiJarParameters = abi_params,
        )

        return struct(
            baseCommandParams = base_command_params,
            abiJarCommand = abi_command,
        )

    # buildifier: disable=uninitialized
    def define_javacd_action(
            category_prefix: str.type,
            actions_identifier: [str.type, None],
            encoded_command: struct.type,
            qualified_name: str.type,
            output_paths: OutputPaths.type,
            classpath_jars_tag: "artifact_tag",
            abi_dir: ["artifact", None],
            target_type: TargetType.type,
            path_to_class_hashes: ["artifact", None],
            debug_port: [int.type, None],
            debug_target: ["label", None],
            extra_jvm_args: [str.type],
            is_creating_subtarget: bool.type = False,
            source_only_abi_compiling_deps: ["JavaClasspathEntry"] = []):
        proto = declare_prefixed_output(actions, actions_identifier, "jar_command.proto.json")

        proto_with_inputs = actions.write_json(proto, encoded_command, with_inputs = True)

        # for javacd we expect java_toolchain.javac to be a dependency. Otherwise, it won't work when we try to debug it.
        expect(type(java_toolchain.javac) == "dependency", "java_toolchain.javac must be of type dependency but it is {}".format(type(java_toolchain.javac)))
        cmd = cmd_args()
        cmd.add(
            java_toolchain.java[RunInfo],
        )

        if debug_port and qualified_name.startswith(base_qualified_name(debug_target)):
            cmd.add(
                "-agentlib:jdwp=transport=dt_socket,server=y,suspend=y,address={}".format(debug_port),
            )

        if len(extra_jvm_args) > 0:
            cmd.add(extra_jvm_args)

        cmd.add(
            "-XX:-MaxFDLimit",
            "-jar",
            java_toolchain.javac[DefaultInfo].default_outputs[0],
        )

        cmd.add(
            "--action-id",
            qualified_name,
            "--command-file",
            proto_with_inputs,
        )
        if target_type == TargetType("library") and should_create_class_abi:
            cmd.add(
                "--full-library",
                output_paths.jar.as_output(),
                "--class-abi-output",
                class_abi_jar.as_output(),
                "--abi-output-dir",
                class_abi_output_dir.as_output(),
            )

        if target_type == TargetType("source_abi") or target_type == TargetType("source_only_abi"):
            cmd.add(
                "--javacd-abi-output",
                output_paths.jar.as_output(),
                "--abi-output-dir",
                abi_dir.as_output(),
            )

        cmd = add_output_paths_to_cmd_args(cmd, output_paths, path_to_class_hashes)

        # TODO(cjhopman): make sure this works both locally and remote.
        event_pipe_out = declare_prefixed_output(actions, actions_identifier, "events.data")

        dep_files = {}
        if not is_creating_subtarget and srcs and (java_toolchain.dep_files == DepFiles("per_jar") or java_toolchain.dep_files == DepFiles("per_class")) and track_class_usage:
            abi_to_abi_dir_map = None
            hidden = []
            if java_toolchain.dep_files == DepFiles("per_class"):
                if target_type == TargetType("source_only_abi"):
                    abi_as_dir_deps = [dep for dep in source_only_abi_compiling_deps if dep.abi_as_dir]
                    abi_to_abi_dir_map = [cmd_args(dep.abi, dep.abi_as_dir, delimiter = " ") for dep in abi_as_dir_deps]
                    hidden = [dep.abi_as_dir for dep in abi_as_dir_deps]
                elif compiling_deps_tset:
                    abi_to_abi_dir_map = compiling_deps_tset.project_as_args("abi_to_abi_dir")
            used_classes_json_outputs = [output_paths.jar_parent.project("used-classes.json")]
            cmd = setup_dep_files(
                actions,
                actions_identifier,
                cmd,
                classpath_jars_tag,
                used_classes_json_outputs,
                abi_to_abi_dir_map,
                hidden = hidden,
            )

            dep_files["classpath_jars"] = classpath_jars_tag

        actions.run(
            cmd,
            env = {
                "BUCK_EVENT_PIPE": event_pipe_out.as_output(),
                "JAVACD_ABSOLUTE_PATHS_ARE_RELATIVE_TO_CWD": "1",
            },
            category = "{}javacd_jar".format(category_prefix),
            identifier = actions_identifier or "",
            dep_files = dep_files,
        )

    library_classpath_jars_tag = actions.artifact_tag()
    command = encode_library_command(output_paths, path_to_class_hashes_out, library_classpath_jars_tag)
    define_javacd_action(
        "",
        actions_identifier,
        command,
        base_qualified_name(label),
        output_paths,
        library_classpath_jars_tag,
        class_abi_output_dir if should_create_class_abi else None,
        TargetType("library"),
        path_to_class_hashes_out,
        java_toolchain.javacd_debug_port,
        java_toolchain.javacd_debug_target,
        java_toolchain.javacd_jvm_args,
        is_creating_subtarget,
    )
    final_jar = prepare_final_jar(actions, actions_identifier, output, output_paths, additional_compiled_srcs, java_toolchain.jar_builder)
    if not is_creating_subtarget:
        class_abi, source_abi, source_only_abi, classpath_abi, classpath_abi_dir = generate_abi_jars(
            actions,
            actions_identifier,
            label,
            abi_generation_mode,
            additional_compiled_srcs,
            is_building_android_binary,
            java_toolchain.class_abi_generator,
            final_jar,
            compiling_deps_tset,
            source_only_abi_deps,
            class_abi_jar = class_abi_jar,
            class_abi_output_dir = class_abi_output_dir,
            encode_abi_command = encode_abi_command,
            define_action = define_javacd_action,
            debug_port = java_toolchain.javacd_debug_port,
            debug_target = java_toolchain.javacd_debug_target,
            extra_jvm_args = java_toolchain.javacd_jvm_args,
        )

        result = make_compile_outputs(
            full_library = final_jar,
            class_abi = class_abi,
            source_abi = source_abi,
            source_only_abi = source_only_abi,
            classpath_abi = classpath_abi,
            classpath_abi_dir = classpath_abi_dir,
            required_for_source_only_abi = required_for_source_only_abi,
            annotation_processor_output = output_paths.annotations,
        )
    else:
        result = make_compile_outputs(
            full_library = final_jar,
            required_for_source_only_abi = required_for_source_only_abi,
            annotation_processor_output = output_paths.annotations,
        )
    return result