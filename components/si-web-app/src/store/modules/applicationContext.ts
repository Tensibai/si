import Bottle from "bottlejs";
import _ from "lodash";
import { Module } from "vuex";
import { SiVuexStore, InstanceStoreContext } from "@/store";
import { ChangeSet } from "@/api/sdf/model/changeSet";
import {
  ApplicationContextDal,
  IGetApplicationContextRequest,
  IGetApplicationContextReplySuccess,
  ICreateChangeSetAndEditSessionRequest,
  ICreateChangeSetAndEditSessionReply,
  IGetChangeSetAndEditSessionRequest,
  IGetChangeSetAndEditSessionReply,
  ICreateEditSessionRequest,
  ICreateEditSessionReply,
  ICancelEditSessionRequest,
  ICancelEditSessionReply,
  ICreateEditSessionAndGetChangeSetRequest,
  ICreateEditSessionAndGetChangeSetReply,
} from "@/api/sdf/dal/applicationContextDal";
import { EditSession } from "@/api/sdf/model/editSession";
import { CurrentChangeSetEvent } from "@/api/partyBus/currentChangeSetEvent";

export interface ApplicationContextStore {
  applicationId: string | null;
  activatedBy: Set<string>;
  applicationName: string | null;
  systemsList: {
    value: string;
    label: string;
  }[];
  currentSystem: string | null;
  openChangeSetsList: {
    value: string;
    label: string;
  }[];
  currentChangeSet: ChangeSet | null;
  currentEditSession: EditSession | null;
  statusBarCtx?: InstanceStoreContext | null;
  editMode: boolean;
}

export const applicationContext: Module<ApplicationContextStore, any> = {
  namespaced: true,
  state(): ApplicationContextStore {
    return {
      applicationId: null,
      activatedBy: new Set(),
      applicationName: null,
      systemsList: [],
      currentSystem: null,
      openChangeSetsList: [],
      currentChangeSet: null,
      currentEditSession: null,
      statusBarCtx: null,
      editMode: false,
    };
  },
  mutations: {
    setCurrentChangeSetAndEditSession(
      state,
      payload: {
        changeSet: ApplicationContextStore["currentChangeSet"];
        editSession: ApplicationContextStore["currentEditSession"];
      },
    ) {
      state.currentChangeSet = payload.changeSet;
      state.currentEditSession = payload.editSession;
    },
    setCurrentChangeSet(
      state,
      payload: ApplicationContextStore["currentChangeSet"],
    ) {
      state.currentChangeSet = payload;
    },
    setCurrentEditSession(
      state,
      payload: ApplicationContextStore["currentEditSession"],
    ) {
      state.currentEditSession = payload;
    },
    updateOpenChangeSetsList(
      state,
      payload: ApplicationContextStore["openChangeSetsList"],
    ) {
      state.openChangeSetsList = _.union(payload, state.openChangeSetsList);
    },
    setApplicationContext(state, payload: IGetApplicationContextReplySuccess) {
      state.applicationName = payload.applicationName;
      state.systemsList = payload.systemsList;
      state.openChangeSetsList = payload.openChangeSetsList;
      state.openChangeSetsList.push({ label: "", value: "" });
    },
    setEditMode(state, payload: ApplicationContextStore["editMode"]) {
      state.editMode = payload;
    },
    addToActivatedBy(state, payload: string) {
      state.activatedBy = state.activatedBy.add(payload);
    },
    removeFromActivatedBy(state, payload: string) {
      state.activatedBy.delete(payload);
    },
    clearCurrentChangeSetAndCurrentEditSession(state) {
      state.currentChangeSet = null;
      state.currentEditSession = null;
    },
    clear(state) {
      state.applicationId = null;
      state.applicationName = null;
      state.systemsList = [];
      state.currentSystem = null;
      state.openChangeSetsList = [];
      state.currentChangeSet = null;
      state.statusBarCtx = undefined;
    },
    setStatusBarCtx(state, payload: ApplicationContextStore["statusBarCtx"]) {
      state.statusBarCtx = payload;
    },
  },
  actions: {
    activate({ commit }, payload: InstanceStoreContext) {
      commit("addToActivatedBy", payload.activateName());
      const bottle = Bottle.pop("default");
      bottle.container.UpdateTracker.register("Entity", payload.dispatchPath());
    },
    deactivate({ commit, state }, payload: InstanceStoreContext) {
      commit("removeFromActivatedBy", payload.activateName());
      if (state.activatedBy.size == 0) {
        commit("clear");
      }
    },
    setStatusBarCtx({ commit }, payload: InstanceStoreContext) {
      commit("setStatusBarCtx", payload);
    },
    async setEditMode(
      { commit, state, dispatch },
      payload: ApplicationContextStore["editMode"],
    ) {
      commit("setEditMode", payload);
      if (state.statusBarCtx) {
        await dispatch(
          state.statusBarCtx.dispatchPath("setEditMode"),
          payload,
          { root: true },
        );
      }
    },
    async loadApplicationContext(
      { commit, state, dispatch },
      request: IGetApplicationContextRequest,
    ) {
      let reply = await ApplicationContextDal.getApplicationContext(request);
      if (!reply.error) {
        commit("setApplicationContext", reply);
        if (state.statusBarCtx) {
          dispatch(
            state.statusBarCtx.dispatchPath("setApplicationName"),
            reply.applicationName,
            { root: true },
          );
        }
      }
      return reply;
    },
    async loadChangeSetAndEditSession(
      { commit, dispatch, state },
      request: IGetChangeSetAndEditSessionRequest,
    ): Promise<IGetChangeSetAndEditSessionReply> {
      let reply = await ApplicationContextDal.getChangeSetAndEditSession(
        request,
      );
      if (!reply.error) {
        commit("setCurrentChangeSetAndEditSession", reply);
        new CurrentChangeSetEvent(reply.changeSet).publish();
      }
      return reply;
    },
    async createEditSessionAndLoadChangeSet(
      { commit, dispatch, state },
      request: ICreateEditSessionAndGetChangeSetRequest,
    ): Promise<ICreateEditSessionAndGetChangeSetReply> {
      let reply = await ApplicationContextDal.createEditSessionAndGetChangeSet(
        request,
      );
      if (!reply.error) {
        commit("setCurrentChangeSetAndEditSession", reply);
        new CurrentChangeSetEvent(reply.changeSet).publish();
      }
      return reply;
    },
    async clearCurrentChangeSetAndCurrentEditSession({
      commit,
      state,
      dispatch,
    }) {
      commit("clearCurrentChangeSetAndCurrentEditSession");
      if (state.statusBarCtx) {
        new CurrentChangeSetEvent(null).publish();
      }
    },
    async createChangeSetAndEditSession(
      { commit, state, dispatch },
      request: ICreateChangeSetAndEditSessionRequest,
    ): Promise<ICreateChangeSetAndEditSessionReply> {
      let reply = await ApplicationContextDal.createChangeSetAndEditSession(
        request,
      );
      if (!reply.error) {
        commit("updateOpenChangeSetsList", [
          {
            label: reply.changeSet.name,
            value: reply.changeSet.id,
          },
        ]);
        commit("setCurrentChangeSet", reply.changeSet);
        if (state.statusBarCtx) {
          new CurrentChangeSetEvent(reply.changeSet).publish();
        }
        commit("setCurrentEditSession", reply.editSession);
      }
      return reply;
    },
    async createEditSession(
      { commit },
      request: ICreateEditSessionRequest,
    ): Promise<ICreateEditSessionReply> {
      let reply = await ApplicationContextDal.createEditSession(request);
      if (!reply.error) {
        commit("setCurrentEditSession", reply.editSession);
      }
      return reply;
    },
    async cancelEditSession(
      { commit },
      request: ICancelEditSessionRequest,
    ): Promise<ICancelEditSessionReply> {
      let reply = await ApplicationContextDal.cancelEditSession(request);
      if (!reply.error) {
        commit("setCurrentEditSession", null);
      }
      return reply;
    },
  },
};

export async function registerApplicationContext(
  applicationContextCtx: InstanceStoreContext,
  statusBarCtx: InstanceStoreContext,
) {
  const bottle = Bottle.pop("default");
  const store: SiVuexStore = bottle.container.Store;
  if (
    store.hasModule(applicationContextCtx.storeName) &&
    !store.hasModule([
      applicationContextCtx.storeName,
      applicationContextCtx.instanceId,
    ])
  ) {
    store.registerModule(
      [applicationContextCtx.storeName, applicationContextCtx.instanceId],
      applicationContext,
    );
    await store.dispatch(
      applicationContextCtx.dispatchPath("setStatusBarCtx"),
      statusBarCtx,
    );
  }
}

export function unregisterApplicationContext(ctx: InstanceStoreContext) {
  const bottle = Bottle.pop("default");
  const store: SiVuexStore = bottle.container.Store;
  if (store.hasModule([ctx.storeName, ctx.instanceId])) {
    store.unregisterModule([ctx.storeName, ctx.instanceId]);
  }
}
