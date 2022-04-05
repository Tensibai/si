extern crate proc_macro;

mod dal_test;

use proc_macro::TokenStream;
use syn::{parse_macro_input, AttributeArgs, ItemFn};

/// A procedural macro which helps to streamline, setup, and manage DAL object-related tests.
///
/// The attribute is intended to replace the default `#[test]` attribute, the augmented
/// `#[tokio::test]` attribute provided by Tokio, and even the `#[test_log::test]` attribute which
/// configures optional tracing/logging output.
///
/// # Examples
///
/// ## Reasonable Default
///
/// Writing a test which begins with:
///
/// * a fully signed up billing account
/// * with an open change set and edit session
/// * with the billing account's workspace read and write tenancy
/// * with the `SystemInit` history actor
///
/// ```ignore
/// use dal::DalContext;
/// use crate::dal::test;
///
/// #[test]
/// async fn good_defaults(ctx: &DalContext<'_, '_>) {
///     // ...
/// }
/// ```
///
/// ## Mutating Default Context
///
/// If the context needs to be mutated in place, it can also be mutably borrowed into the test:
///
/// ```ignore
/// use dal::DalContext;
/// use crate::dal::test;
///
/// #[test]
/// async fn good_defaults(ctx: &mut DalContext<'_, '_>) {
///     ctx.update_to_universal_head();
///     // ...
/// }
/// ```
///
/// ## Owned Default Context
///
/// Finally, it might be easier to have full ownership of the context, in which case it can be
/// moved into the test function with:
///
/// ```ignore
/// use dal::DalContext;
/// use crate::dal::test;
///
/// #[test]
/// async fn good_defaults(ctx: DalContext<'_, '_>) {
///     // ...
/// }
/// ```
///
/// # Owned Types
///
/// The following types can be used as test function arguments as owned types, provided by the
/// internal setup code generated by the attribute:
///
/// * `bid: BillingAccountId`: the billing account ID of the billing account created for this test
/// * `nba: BillingAccountSignup`: the full "new-billing-account" data structure, created for this
///   test
/// * `ctx: DalContext<'_, '_>`: a DAL context for the created billing account with an open change
///   set and edit session
/// * `builder: DalContextBuilder`: the builder to create DAL context objects
/// * `DalContextHead(ctx): DalContextHead<'_, '_>`: a DAL context for a workspace in the billing
///    account
///    which is not in a change set nor an edit session. `ctx` is **owned**.
/// * `DalContextHeadRef(ctx): DalContextHeadRef<'_, '_, '_>`: a reference to a DAL context for a
///    workspace in the billing account which is not in a change set nor an edit session
/// * `DalContextHeadMutRef(ctx): DalContextHeadMutRef<'_, '_, '_>`: a mutable reference to a DAL
///    context for a workspace in the billing account which is not in a change set nor an edit
///    session
/// * `DalContextUniversalHead(ctx): DalContextUniversalHead<'_, '_>`: a DAL context with universal
///    read/write tenancies and a head visibility. `ctx` is **owned**.
/// * `DalContextUniversalHeadRef(ctx): DalContextUniversalHeadRef<'_, '_, '_>`: a reference to a
///    DAL context with universal read/write tenancies and a head visibility
/// * `DalContextUniversalHeadMutRef(ctx): DalContextUniversalHeadMutRef<'_, '_, '_>`: a mutable
///    reference to a DAL context with universal read/write tenancies and a head visibility
/// * `oid: OrganizationId`: the organization ID of the billing account created for this test
/// * `services_ctx: ServicesContext`: a services context object, used to create DAL contexts
/// * `handle: ShutdownHandle`: the shutdown handle for the Veritech server running alongside each
///    test
/// * `starter: TransactionsStarter`: the type that owns new connections used to start a set of
///    transactions
/// * `wid: WorkspaceId`: the workspace ID of the billing account created for this test
///
/// # Referenced/Borrowed Types
///
/// The following types can be used as test function arguments as borrowed types, provided by the
/// internal setup code generated by the attribute:
///
/// * `nba: &BillingAccountSignup`: a reference to the full "new-billing-account" data structure,
///    created for this test
/// * `ctx: &DalContext<'_, '_>`: a reference to the the default DAL context
/// * `ctx: &mut DalContext<'_, '_>`: a mutable reference to the the default DAL context
/// * `builder: DalContextBuilder`: a reference to the builder to create DAL context objects
/// * `jwt_secret_key: &JwtSecretKey`: a reference to the key used to decrypt the JWT signing key
///    from the database.
/// * `services_ctx: &ServicesContext`: a reference to a services context object, used to create
///    DAL contexts
///
/// # Customized Tokio Runtime
///
/// The attribute uses a similar strategy to the stock `#[tokio::test]` attribute, except that this
/// attribute runs a threaded runtime as opposed to the current thread runtime, allowing for future
/// stack size cusomization. Currently the runtime uses a default thread stack size of **3** times
/// the system default (implementation constant is located in `src/dal_test.rs` from
/// `RT_DEFAULT_THREAD_STACK_SIZE`).
///
/// # Optional and Configurable Logging Output for Tests
///
/// As with the `test-env-log` and `test-log` crates, this attribute also sets up tracing support
/// to log to console in the presence of certain environment variables.
///
/// To enable logging output, the `SI_TEST_LOG` environment variable must be set, and if you wish
/// to see all logging output, even when the tests pass, then you must also use the `--nocapture`
/// flag with `cargo test`. For example:
///
/// ```ignore
/// env SI_TEST_LOG=info cargo test -- --nocapture
/// ```
///
/// Additionally, tracing spans can be enabled and configured via the `SI_TEST_LOG_SPAN_EVENTS`
/// environment variable. For example, to see the "new" and "close" span events in the logging
/// output:
///
/// ```ignore
/// env SI_TEST_LOG=info SI_TEST_LOG_SPAN_EVENTS=new,close cargo test -- --nocapture
/// ```
/// Note that for span events, the following are valid:
///
/// * `new`
/// * `enter`
/// * `exit`
/// * `close`
/// * `active`
/// * `full`
///
/// The implementation for tracing is located in `src/dal_test.rs` in the `expand_tracing_init()`
/// function.
#[proc_macro_attribute]
pub fn dal_test(attr: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as AttributeArgs);
    let item = parse_macro_input!(input as ItemFn);
    dal_test::expand(item, args).into()
}