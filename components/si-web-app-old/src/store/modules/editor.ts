import Vue from "vue";
import { Module } from "vuex";
import _ from "lodash";

import { Entity } from "@/api/sdf/model/entity";
import { System } from "@/api/sdf/model/system";
import { Secret } from "@/api/sdf/model/secret";
import { Resource } from "@/api/sdf/model/resource";
import {
  ChangeSet,
  ChangeSetStatus,
  ChangeSetParticipant,
} from "@/api/sdf/model/changeSet";
import { EventLog } from "@/api/sdf/model/eventLog";
import { Event } from "@/api/sdf/model/event";
import { OutputLine } from "@/api/sdf/model/outputLine";
import { EditSession } from "@/api/sdf/model/editSession";
import { IOpRequest, OpEntitySet } from "@/api/sdf/model/ops";
import { User } from "@/api/sdf/model/user";
import {
  Node,
  NodeKind,
  NodeObject,
  Position,
  RegistryProperty,
} from "@/api/sdf/model/node";
import { Edge } from "@/api/sdf/model/edge";
import { RootStore } from "@/store";
import router from "@/router/index";
import { DiffResult } from "@/utils/diff";

export interface ActionRestore {
  applicationId: string;
}

export interface ActionSetCurrent {
  id: string;
}

export interface ActionSetSystem {
  id: string;
}

export interface ActionSetChangeSet {
  id: string | undefined;
}

export interface ActionSetEditSession {
  id: string | undefined;
}

export interface ActionSetNode {
  id: string | undefined;
}

export interface ActionChangeSetCreate {
  name: string;
}

export interface ActionEntityAction {
  action: string;
  nodeId?: string;
}

export interface ActionNodeCreate {
  kind: NodeKind;
  objectType: string;
  configuredByNodeId?: string;
}

export interface ActionSetNodePosition {
  nodeId: string;
  position: Position;
}

export interface IConnectionPosition {
  sourceNodePosition: {
    nodeId: string;
    x: number;
    y: number;
  };
  destinationNodePosition: {
    nodeId: string;
    x: number;
    y: number;
  };
}

export interface ConfiguresConnection {
  sourceNodeId: string;
  destinationNodeId: string;
}

export interface EventBarItem {
  id: string;
  event: Event;
  logs: EventLog[];
  output: {
    [id: string]: OutputLine;
  };
}

export interface EditorStore {
  mode: "view" | "edit";
  context: string;
  mouseTrackSelection: string | undefined;
  isSaving: boolean;
  editSaveError: undefined | Error;
  changeSetsOpen: ChangeSet[];
  changeSet: ChangeSet | undefined;
  changeSetParticipantCount: number;
  editSession: EditSession | undefined;
  application: Entity | undefined;
  system: System | undefined;
  systems: System[];
  nodes: Node[];
  objects: {
    [key: string]: NodeObject;
  };
  edges: Edge[];
  node: Node | undefined;
  directSuccessors: Node[];
  newConfiguresInputTypes: { value: string | null; label: string }[];
  propertyList: RegistryProperty[];
  secretList: Secret[] | undefined;
  secretName: string | undefined;
  editObject: Entity | undefined;
  diff: DiffResult;
  eventBar: EventBarItem[];
  resources: Resource[];
  currentResource: Resource | undefined;
}

export let SET_POSITION_FUNCTIONS: Record<string, any> = {};

export const editor: Module<EditorStore, RootStore> = {
  namespaced: true,
  state: {
    context: "none",
    mode: "view",
    mouseTrackSelection: undefined,
    isSaving: false,
    editSaveError: undefined,
    changeSetsOpen: [],
    changeSet: undefined,
    changeSetParticipantCount: 0,
    editSession: undefined,
    application: undefined,
    system: undefined,
    systems: [],
    nodes: [],
    objects: {},
    edges: [],
    node: undefined,
    directSuccessors: [],
    newConfiguresInputTypes: [],
    propertyList: [],
    secretList: undefined,
    secretName: undefined,
    editObject: undefined,
    diff: {
      entries: [],
      count: 0,
    },
    eventBar: [],
    resources: [],
    currentResource: undefined,
  },
  mutations: {
    currentResource(state, payload: Resource | undefined) {
      state.currentResource = payload;
    },
    directSuccessors(state, payload: Node[]) {
      state.directSuccessors = payload;
    },
    mouseTrackSelection(state, payload: string | undefined) {
      state.mouseTrackSelection = payload;
    },
    context(state, payload: string) {
      state.context = payload;
    },
    updateEventBar(state, { event, logs }: { event: Event; logs: EventLog[] }) {
      state.eventBar = _.take(
        _.orderBy(
          _.unionBy(
            [
              {
                id: event.id,
                event,
                logs: _.orderBy(logs, ["unixTimestamp", "asc"]),
                output: {},
              },
            ],
            state.eventBar,
            "id",
          ),
          ["event.startUnixTimestamp"],
          ["desc"],
        ),
        60,
      );
    },
    updateObjects(state, payload: NodeObject) {
      Vue.set(state.objects, payload.nodeId, payload);
    },
    updateResources(state, payload: Resource) {
      state.resources = _.orderBy(
        _.unionBy([payload], state.resources, "id"),
        ["id"],
        ["desc"],
      );
    },
    setResources(state, payload: Resource[]) {
      state.resources = payload;
    },
    node(state, payload: Node | undefined) {
      state.node = payload;
      router
        .replace({
          query: Object.assign({}, router.currentRoute.query, {
            nodeId: payload?.id,
          }),
        })
        .catch(_ => {});
    },
    updateNodes(state, payload: Node) {
      state.nodes = _.unionBy([payload], state.nodes, "id");
    },
    setNodes(state, payload: Node[]) {
      state.nodes = payload;
    },
    setSecretList(state, payload: Secret[] | undefined) {
      state.secretList = payload;
    },
    setSecretName(state, payload: string | undefined) {
      state.secretName = payload;
    },
    setPropertyList(state, payload: RegistryProperty[]) {
      state.propertyList = payload;
    },
    setEditObject(state, payload: Entity | undefined) {
      state.editObject = payload;
    },
    setObjects(state, payload: EditorStore["objects"]) {
      state.objects = payload;
    },
    application(state, payload: Entity | undefined) {
      state.application = payload;
    },
    system(state, payload: System | undefined) {
      state.system = payload;
      router
        .replace({
          query: Object.assign(
            {},
            { ...router.currentRoute.query },
            {
              systemId: payload?.id,
            },
          ),
        })
        .catch(_ => {});
    },
    setSystems(state, payload: System[]) {
      state.systems = payload;
    },
    setIsSaving(state, saving: boolean) {
      state.isSaving = saving;
    },
    setEditSaveError(state, error: Error) {
      state.editSaveError = error;
    },
    setMode(state, mode: EditorStore["mode"]) {
      state.mode = mode;
      router
        .replace({
          query: Object.assign({}, router.currentRoute.query, {
            mode: mode,
          }),
        })
        .catch(_ => {});
    },
    setEdges(state, edges: Edge[]) {
      state.edges = edges;
    },
    changeSetsOpenAdd(state, payload: ChangeSet) {
      state.changeSetsOpen = _.unionBy([payload], state.changeSetsOpen, "id");
    },
    changeSetsOpenRemove(state, payload: ChangeSet) {
      let changeSetsOpen = _.cloneDeep(state.changeSetsOpen);
      _.remove(changeSetsOpen, ["id", payload.id]);
      state.changeSetsOpen = changeSetsOpen;
    },
    changeSetParticipantCount(state, payload: number) {
      state.changeSetParticipantCount = payload;
    },
    changeSet(state, payload: ChangeSet | undefined) {
      state.changeSet = payload;
      router
        .replace({
          query: Object.assign({}, router.currentRoute.query, {
            changeSetId: payload?.id,
          }),
        })
        .catch(_ => {});
    },
    editSession(state, payload: EditSession | undefined) {
      state.editSession = payload;
      router
        .replace({
          query: Object.assign({}, router.currentRoute.query, {
            editSessionId: payload?.id,
          }),
        })
        .catch(_ => {});
    },
    diff(state, payload: DiffResult) {
      state.diff = payload;
    },
    newConfiguresInputTypes(
      state,
      payload: { value: string | null; label: string }[],
    ) {
      state.newConfiguresInputTypes = payload;
    },
    clear(state) {
      state.context = "none";
      state.mode = "view";
      state.mouseTrackSelection = undefined;
      state.isSaving = false;
      state.editSaveError = undefined;
      state.changeSetsOpen = [];
      state.changeSet = undefined;
      state.editSession = undefined;
      state.application = undefined;
      state.system = undefined;
      state.systems = [];
      state.nodes = [];
      state.objects = {};
      state.node = undefined;
      state.directSuccessors = [];
      state.edges = [];
      state.propertyList = [];
      state.secretList = undefined;
      state.secretName = undefined;
      state.editObject = undefined;
      state.changeSetParticipantCount = 0;
      state.diff = {
        entries: [],
        count: 0,
      };
      state.eventBar = [];
      state.resources = [];
      state.currentResource = undefined;
      SET_POSITION_FUNCTIONS = {};
    },
  },
  getters: {
    // prettier-ignore
    propertiesListRepeated: (state) => (entityProperty: RegistryProperty, index: number): RegistryProperty[] => {
      let node = state.node;
      let changeSet = state.changeSet;
      if (node) {
        return node.propertyListRepeated(entityProperty, index, changeSet?.id);
      } else {
        throw new Error("no node object for repeated property list! bug!");
      }
    },
    codeProperty(state): undefined | RegistryProperty {
      let propertiesList = state.propertyList;
      for (const prop of propertiesList) {
        if (prop.kind == "code") {
          return prop;
        }
      }
      return undefined;
    },
    nodeList(state): EditorStore["nodes"] {
      return _.filter(state.nodes, n => {
        if (state.objects[n.id]) {
          if (
            state.objects[n.id].head &&
            state.objects[n.id].siStorable.deleted
          ) {
            return false;
          } else {
            return true;
          }
        } else {
          return false;
        }
      });
    },
    positions(state, getters): IConnectionPosition[] {
      const result: IConnectionPosition[] = [];
      if (state.context) {
        for (let edge of state.edges) {
          let sourceNode = _.find(getters["nodeList"], [
            "id",
            edge.tailVertex.nodeId,
          ]);
          let sourceNodePosition:
            | { nodeId: string; x: number; y: number }
            | undefined;
          let destinationNodePosition:
            | { nodeId: string; x: number; y: number }
            | undefined;
          if (sourceNode) {
            sourceNodePosition = {
              nodeId: sourceNode.id,
              ...Node.upgrade(sourceNode).position(state.context),
            };
          }
          let destNode = _.find(getters["nodeList"], [
            "id",
            edge.headVertex.nodeId,
          ]);
          if (destNode) {
            destinationNodePosition = {
              nodeId: destNode.id,
              ...Node.upgrade(destNode).position(state.context),
            };
          }
          if (sourceNodePosition && destinationNodePosition) {
            result.push({ sourceNodePosition, destinationNodePosition });
          }
        }
      }
      return result;
    },
  },
  actions: {
    async deleteConfigures({ state }, payload: Node) {
      let selectedNode = state.node;
      if (selectedNode) {
        await selectedNode.deleteSuccessor(payload);
      }
    },
    async createNewConfigures({ state, dispatch }, payload: string) {
      let node = state.node;
      if (node) {
        if (payload.includes(" ")) {
          let parts = payload.split(" ");
          let successorNode = await Node.get({ id: parts[1] });
          await successorNode.configuredBy(node.id);
        } else {
          await dispatch("nodeCreate", {
            kind: NodeKind.Entity,
            objectType: payload,
            configuredByNodeId: node.id,
          });
        }
      }
    },
    async createNewConfiguresConnection(
      { state, dispatch },
      payload: ConfiguresConnection,
    ) {
      var predecessorNode;
      var successorNode;

      if (payload.sourceNodeId) {
        predecessorNode = await Node.get({ id: payload.sourceNodeId });
      }
      if (payload.destinationNodeId) {
        successorNode = await Node.get({ id: payload.destinationNodeId });
      }

      if (predecessorNode && successorNode) {
        await successorNode.configuredBy(predecessorNode.id);
      }
    },
    async setNodePosition({ state, commit }, payload: ActionSetNodePosition) {
      let node = _.find(state.nodes, ["id", payload.nodeId]);
      let context = state.context;
      if (node) {
        let unode = Node.upgrade(_.cloneDeep(node));
        unode.positions[state.context] = payload.position;
        commit("updateNodes", unode);
      }
      if (SET_POSITION_FUNCTIONS[payload.nodeId]) {
        SET_POSITION_FUNCTIONS[payload.nodeId](
          payload.nodeId,
          payload.position,
          context,
        );
      } else {
        SET_POSITION_FUNCTIONS[payload.nodeId] = _.debounce(
          async (nodeId: string, position: Position, context: string) => {
            let node = await Node.get({ id: nodeId });
            await node.setPosition(context, position);
          },
          1000,
        );
        SET_POSITION_FUNCTIONS[payload.nodeId](
          payload.nodeId,
          payload.position,
          context,
        );
      }
    },
    async sendAction({ dispatch }, payload: { action: string }) {
      if (payload.action == "delete") {
        await dispatch("entityDelete", { cascade: true });
      } else {
        await dispatch("entityAction", payload);
      }
    },
    async entityAction({ state, rootGetters }, payload: ActionEntityAction) {
      let organization = rootGetters["organization/current"];
      let workspace = rootGetters["workspace/current"];
      let changeSet = state.changeSet;
      let editSession = state.editSession;
      let system = state.system;
      let node: Node | undefined;
      if (payload.nodeId) {
        node = await Node.get({ id: payload.nodeId });
      } else {
        node = state.node;
      }
      if (
        organization &&
        workspace &&
        changeSet &&
        editSession &&
        node &&
        system
      ) {
        let op = {
          entityAction: {
            action: payload.action,
            systemId: system.id,
          },
        };
        let req = {
          op,
          organizationId: organization.id,
          workspaceId: workspace.id,
          changeSetId: changeSet.id,
          editSessionId: editSession.id,
        };
        await OpEntitySet.create(node.id, req);
      }
    },
    async entityDelete(
      { state, rootGetters },
      payload: IOpRequest["entityDelete"],
    ) {
      let organization = rootGetters["organization/current"];
      let workspace = rootGetters["workspace/current"];
      let changeSet = state.changeSet;
      let editSession = state.editSession;
      let node = state.node;
      if (organization && workspace && changeSet && editSession && node) {
        let cascade = true;
        if (payload?.cascade === false) {
          cascade = false;
        }
        let op = {
          entityDelete: {
            cascade,
          },
        };
        let req = {
          op,
          organizationId: organization.id,
          workspaceId: workspace.id,
          changeSetId: changeSet.id,
          editSessionId: editSession.id,
        };
        await OpEntitySet.create(node.id, req);
      }
    },
    async entityNameSet(
      { state, rootGetters },
      payload: IOpRequest["nameSet"],
    ) {
      let organization = rootGetters["organization/current"];
      let workspace = rootGetters["workspace/current"];
      let changeSet = state.changeSet;
      let editSession = state.editSession;
      let node = state.node;
      if (organization && workspace && changeSet && editSession && node) {
        let op = {
          nameSet: payload,
        };
        let req = {
          op,
          organizationId: organization.id,
          workspaceId: workspace.id,
          changeSetId: changeSet.id,
          editSessionId: editSession.id,
        };
        await OpEntitySet.create(node.id, req);
      }
    },
    async entitySet({ state, rootGetters }, payload: IOpRequest["entitySet"]) {
      let organization = rootGetters["organization/current"];
      let workspace = rootGetters["workspace/current"];
      let changeSet = state.changeSet;
      let editSession = state.editSession;
      let node = state.node;
      if (organization && workspace && changeSet && editSession && node) {
        let op = {
          entitySet: payload,
        };
        let req = {
          op,
          organizationId: organization.id,
          workspaceId: workspace.id,
          changeSetId: changeSet.id,
          editSessionId: editSession.id,
        };
        await OpEntitySet.create(node.id, req);
      }
    },
    async setMouseTrackSelection({ commit }, payload: string | undefined) {
      commit("mouseTrackSelection", payload);
    },
    async context({ commit, state }) {
      let contextState = ["application"];

      if (state.application) {
        contextState.push(state.application.id);
      }
      if (state.system) {
        contextState.push(state.system.id);
      }
      commit("context", contextState.join("."));
    },
    modeSwitch({ commit, state }) {
      if (state.mode == "view") {
        commit("setMode", "edit");
      } else {
        commit("setMode", "view");
      }
    },
    async setNode({ commit, dispatch }, payload: ActionSetNode) {
      if (payload?.id) {
        // @ts-ignore
        let node = await Node.get(payload);
        await dispatch("node", node);
      } else {
        commit("node", undefined);
      }
    },
    async loadEditObject({ state, commit }) {
      let node = state.node;
      if (node) {
        const secretList = await node.secretList(state.changeSet?.id);
        const propertyList = await node.propertyList(state.changeSet?.id);
        const editObject = await node.displayObject(state.changeSet?.id);
        commit("setSecretList", secretList);
        commit("setPropertyList", propertyList);
        commit("setEditObject", editObject);
      }
    },
    async node({ commit, state }, payload: Node | undefined) {
      commit("node", payload);
      if (payload) {
        const secretList = await payload.secretList(state.changeSet?.id);
        const propertyList = await payload.propertyList(state.changeSet?.id);
        const editObject = (await payload.displayObject(
          state.changeSet?.id,
        )) as Entity;
        const secretName = await editObject.secretName();
        commit("setSecretList", secretList);
        commit("setSecretName", secretName);
        commit("setPropertyList", propertyList);
        commit("setEditObject", editObject);
        let directSuccessors = await payload.directSuccessors();
        commit("directSuccessors", directSuccessors);
        let changeSetId = state.changeSet?.id;
        let inputTypes = await payload.inputTypes(changeSetId);
        inputTypes = _.filter(inputTypes, t => {
          for (const successor of directSuccessors) {
            if (t.value == `${successor.objectType} ${successor.id}`) {
              return false;
            }
          }
          return true;
        });
        commit("newConfiguresInputTypes", inputTypes);
        let nodeResource = _.find(state.resources, ["nodeId", payload.id]);
        if (nodeResource) {
          commit("currentResource", nodeResource);
        } else {
          commit("currentResource", undefined);
        }
        if (editObject.siStorable.typeName == "entity") {
          // @ts-ignore
          let diffResult = await editObject.diff();
          commit("diff", diffResult);
        }
      } else {
        commit("setSecretList", undefined);
        commit("setSecretName", undefined);
        commit("setPropertyList", []);
        commit("setEditObject", undefined);
        commit("directSuccessors", []);
        commit("newConfiguresInputTypes", []);
        commit("currentResource", undefined);
        commit("diff", {
          entries: [],
          count: 0,
        });
      }
    },
    async changeSetExecute({ commit, state }) {
      const changeSet = state.changeSet;
      if (changeSet) {
        await ChangeSet.upgrade(changeSet).execute({ hypothetical: false });
        commit("setMode", "view");
      }
    },
    async deployApplication({ commit, state }) {
      const changeSet = state.changeSet;
      if (!changeSet) {
        console.log(
          "changeSetExecuteWithAction called with no current changeSet--mistake??",
        );
        return;
      }

      const nodeId = state.application?.nodeId;
      if (!nodeId) {
        console.log(
          "changeSetExecuteWithAction called with no current application--mistake??",
        );
        return;
      }

      const systemId = state.system?.id;
      if (!systemId) {
        console.log(
          "changeSetExecuteWithAction called with no current system--mistake??",
        );
        return;
      }

      const editSessionId = state.editSession?.id;
      if (!editSessionId) {
        console.log(
          "changeSetExecuteWithAction called with no current edit session--mistake??",
        );
        return;
      }

      await ChangeSet.upgrade(changeSet).executeWithAction({
        nodeId,
        action: "deploy",
        systemId,
        editSessionId,
      });
      commit("setMode", "view");
    },
    async setEditSession({ commit }, payload: ActionSetEditSession) {
      if (payload.id) {
        // @ts-ignore
        let editSession = await EditSession.get(payload);
        commit("editSession", editSession);
      } else {
        commit("editSession", undefined);
      }
    },
    async setChangeSet(
      { commit, state, dispatch },
      payload: ActionSetChangeSet,
    ) {
      if (payload.id) {
        // @ts-ignore
        let changeSet = await ChangeSet.get(payload);
        commit("changeSet", changeSet);
        let csp = await ChangeSetParticipant.forChangeSet(changeSet.id);
        commit("changeSetParticipantCount", csp.length);
      } else {
        commit("changeSet", undefined);
        commit("changeSetParticipantCount", 0);
      }

      let application = state.application;
      if (application) {
        let applicationNode = await Node.get({ id: application.nodeId });
        let successors = await applicationNode.successors();
        let objects: Record<string, NodeObject> = {};
        let resources: Resource[] = [];
        for (let n of successors) {
          try {
            let obj = await n.displayObject(payload.id);
            objects[obj.nodeId] = obj;
            if (state.system) {
              let resource = await Resource.getByEntityIdAndSystemId(
                n.id,
                state.system.id,
              );
              if (resource) {
                resources.push(resource);
              }
            }
          } catch {}
        }
        commit("setResources", resources);
        commit("setObjects", objects);
        if (state.node && objects[state.node?.id]) {
          dispatch("node", state.node);
        } else {
          dispatch("node", undefined);
        }
      }
    },
    async setApplication(
      { commit, state, dispatch },
      payload: ActionSetCurrent,
    ) {
      let application = await Entity.get_head(payload);
      let applicationNode = await Node.get({ id: application.nodeId });
      let successors = await applicationNode.successors();
      let systems = await application.systems();
      commit("application", application);
      commit("setSystems", systems);
      commit("system", systems[0]);
      let objects: Record<string, NodeObject> = {};
      let resources: Resource[] = [];
      for (let n of successors) {
        try {
          let obj = await n.displayObject(state.changeSet?.id);
          objects[obj.nodeId] = obj;
          let resource = await Resource.getByEntityIdAndSystemId(
            n.id,
            systems[0].id,
          );
          if (resource) {
            resources.push(resource);
          }
        } catch {
          //console.log("node object not included in this changeset");
        }
      }
      commit("setObjects", objects);
      commit("setNodes", successors);
      let edges = await applicationNode.successorEdges();
      commit("setEdges", edges);
      commit("setResources", resources);
      await dispatch("context");
    },
    async setSystem({ commit, dispatch }, payload: ActionSetSystem) {
      let system = await System.get(payload);
      commit("system", system);
      await dispatch("context");
    },
    async editSessionCancel({ state }) {
      let editSession = state.editSession;
      if (editSession) {
        await editSession.cancel();
      }
    },
    async editSessionCreate({ commit, rootGetters, state }) {
      let workspace = rootGetters["workspace/current"];
      let organization = rootGetters["organization/current"];
      let user: User = rootGetters["user/current"];
      let changeSet = state.changeSet;
      if (!changeSet) {
        throw new Error("cannot start an edit session without a change set!");
      }
      let currentDate = new Date();
      let name = `${user.name} ${currentDate.toISOString()}`;
      let editSession = await EditSession.create(changeSet.id, {
        name,
        workspaceId: workspace.id,
        organizationId: organization.id,
      });
      commit("editSession", editSession);
    },
    async changeSetCreate(
      { commit, rootGetters, dispatch },
      payload: ActionChangeSetCreate,
    ): Promise<ChangeSet> {
      let workspace = rootGetters["workspace/current"];
      let organization = rootGetters["organization/current"];
      let changeSet = await ChangeSet.create({
        name: payload.name,
        workspaceId: workspace.id,
        organizationId: organization.id,
      });
      commit("changeSet", changeSet);
      await dispatch("editSessionCreate");
      await dispatch("modeSwitch");
      return changeSet;
    },
    async nodeCreate(
      { commit, dispatch, rootGetters, state },
      payload: ActionNodeCreate,
    ): Promise<Node> {
      let workspace = rootGetters["workspace/current"];
      let organization = rootGetters["organization/current"];
      let changeSetId = state.changeSet?.id;
      let editSessionId = state.editSession?.id;
      let system = state.system;
      let application = state.application;
      if (!changeSetId || !editSessionId || !system || !application) {
        throw new Error(
          `invalid editor state; cannot add node: cs ${changeSetId} es ${editSessionId} s ${system} a ${application}`,
        );
      }
      let configuredByNodeId = payload.configuredByNodeId;
      if (!configuredByNodeId) {
        configuredByNodeId = application.nodeId;
      }

      const node = await Node.create({
        kind: payload.kind,
        objectType: payload.objectType,
        organizationId: organization.id,
        workspaceId: workspace.id,
        changeSetId,
        editSessionId,
        systemIds: [system.id],
      });
      const edge = await node.configuredBy(configuredByNodeId);
      if (state.changeSet) {
        await state.changeSet.execute({ hypothetical: true });
      }
      const object = await node.displayObject(changeSetId);
      commit("updateObjects", object);
      commit("updateNodes", node);
      commit("currentResource", undefined);
      await dispatch("node", node);
      commit("mouseTrackSelection", node.id);
      return node;
    },
    async syncCurrentResource({ state }) {
      let systemId = state.system?.id;
      let changeSetId = state.changeSet?.id;
      if (state.node && systemId && changeSetId) {
        await state.node.syncResource(systemId, changeSetId);
      }
    },
    async syncResource({ state }) {
      if (state.node && state.system) {
        let node = state.node;
        await node.syncResource(state.system.id, state.changeSet?.id);
        // TODO: We don't commit this, because we won't have it yet - it's
        // a fully async operation. Gotta come in through the other side.
        //commit("updateResources", resource);
      }
    },
    async syncResources({ state, getters }) {
      if (state.application && state.system) {
        let nodeList: Node[] = getters["nodeList"];
        for (const node of nodeList) {
          await node.syncResource(state.system.id, state.changeSet?.id);
        }
      }
    },
    async fromNode({ commit, state }, payload: Node) {
      if (state.application) {
        let appNode = await state.application.node();
        let successors = await appNode.successors();
        if (_.find(successors, ["id", payload.id])) {
          commit("updateNodes", payload);
          if (state.node?.id == payload.id) {
            commit("node", payload);
          }
          if (state.system) {
            let resource = await Resource.getByEntityIdAndSystemId(
              payload.id,
              state.system.id,
            );
            commit("updateResources", resource);
          }
        }
      }
    },
    async fromEntity({ commit, state }, payload: Entity) {
      if (state.application) {
        let application = state.application;
        let appNode = await Node.get({ id: application.nodeId });
        let successors = await appNode.successors();
        if (_.find(successors, ["id", payload.nodeId])) {
          let changeSet = state.changeSet;
          if (changeSet) {
            if (
              payload.siChangeSet.changeSetId == changeSet.id &&
              payload.head == false
            ) {
              commit("updateObjects", payload);
              if (state.editObject?.id == payload.id) {
                let diffResult = await payload.diff();
                commit("diff", diffResult);
              }
            }
          } else {
            if (payload.head == true) {
              commit("updateObjects", payload);
            }
          }
        }
      }
    },
    async fromEdge({ commit, state }, payload: Edge) {
      let application = state.application;
      let updatedEdges = false;
      if (application) {
        let appNode = await Node.get({ id: application.nodeId });
        let successors = await appNode.successors();
        if (
          _.find(successors, ["id", payload.tailVertex.nodeId]) ||
          appNode.id == payload.tailVertex.nodeId
        ) {
          updatedEdges = true;
          let changeSetId = state.changeSet?.id;
          let node = await Node.get({ id: payload.headVertex.nodeId });

          console.log("you are about to break", node);
          let entity = await node.displayObject(changeSetId);
          commit("updateNodes", node);
          if (state.node?.id == node.id) {
            commit("node", node);
          }
          commit("updateObjects", entity);
          let nSuccessors = await node.successors();
          for (let ns of nSuccessors) {
            let ne = await ns.displayObject(changeSetId);
            commit("updateNodes", ns);
            if (state.node?.id == ns.id) {
              commit("node", ns);
            }
            commit("updateObjects", ne);
          }
        }
      }
      if (updatedEdges && application) {
        let appNode = await Node.get({ id: application.nodeId });
        let edges = await appNode.successorEdges();
        commit("setEdges", edges);
      }
    },
    fromChangeSet({ commit, dispatch, state }, payload: ChangeSet) {
      if (payload.status == ChangeSetStatus.Open) {
        //console.log("updating from change set", { payload });
        commit("changeSetsOpenAdd", payload);
      } else {
        if (state.changeSet?.id == payload.id) {
          // console.log("removing from change set", { payload });
          dispatch("setChangeSet", { id: undefined });
        }
        commit("changeSetsOpenRemove", payload);
      }
    },
    fromEditSession({ state, commit }, payload: EditSession) {
      if (state.editSession?.id == payload.id) {
        if (!_.isEqual(state.editSession, payload)) {
          commit("editSession", payload);
        }
      }
    },
    async fromEvent({ commit }, payload: Event) {
      await payload.loadOwner();
      const eventLogs = await EventLog.listForEvent(payload.id);
      commit("updateEventBar", { event: payload, logs: eventLogs });
    },
    fromEventLog({ commit }, payload: EventLog) {
      // TODO: We should only show relevant event logs!
      //commit("updateEventLogs", payload);
    },
    fromResource({ commit, state, getters }, payload: Resource) {
      let nodeList = getters["nodeList"];
      if (_.find(nodeList, ["id", payload.nodeId])) {
        commit("updateResources", payload);
      }
      if (state.currentResource?.id == payload.id) {
        commit("currentResource", payload);
      } else if (
        payload.systemId == state.system?.id &&
        payload.nodeId == state.node?.id
      ) {
        commit("currentResource", payload);
      }
    },
    async restore({ dispatch, commit }, payload: ActionRestore) {
      if (router.currentRoute.query["changeSetId"]) {
        await dispatch("setChangeSet", {
          id: router.currentRoute.query["changeSetId"],
        });
      }
      await dispatch("setApplication", {
        id: payload.applicationId,
      });
      if (router.currentRoute.query["systemId"]) {
        await dispatch("setSystem", {
          id: router.currentRoute.query["systemId"],
        });
      }
      if (router.currentRoute.query["nodeId"]) {
        await dispatch("setNode", {
          id: router.currentRoute.query["nodeId"],
        });
      }
      if (router.currentRoute.query["editSessionId"]) {
        await dispatch("setEditSession", {
          id: router.currentRoute.query["editSessionId"],
        });
      }
      if (router.currentRoute.query["mode"]) {
        commit("setMode", router.currentRoute.query["mode"]);
      }
      await dispatch("loadEditObject");
    },
    async clear({ commit }) {
      commit("clear");
    },
  },
};