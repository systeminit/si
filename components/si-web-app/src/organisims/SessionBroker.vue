<template>
  <div />
</template>

<script lang="ts">
import Vue from "vue";
import {
  workspace$,
  changeSet$,
  editSession$,
  user$,
  organization$,
  system$,
  billingAccount$,
  editMode$,
  deploymentSchematicSelectNode$,
  schematicSelectNode$,
  panelTypeChanges$,
  PanelTypeChange,
  restorePanelTypeChanges$,
  schematicPanelKind$,
  restoreSchematicPanelKind$,
  SchematicPanelState,
  attributePanelState$,
  AttributePanelState,
  restoreAttributePanelState$,
} from "@/observables";
import { tap } from "rxjs/operators";
import _ from "lodash";

export interface Data {
  saveAllowed: boolean;
}

export default Vue.extend({
  name: "SessionBroker",
  data(): Record<string, any> {
    return {
      saveAllowed: false,
    };
  },
  subscriptions(this: any): Record<string, any> {
    return {
      user: user$.pipe(tap(v => this.saveObservable("user$", v))),
      billingAccount: billingAccount$.pipe(
        tap(v => this.saveObservable("billingAccount$", v)),
      ),
      organization: organization$.pipe(
        tap(v => this.saveObservable("organization$", v)),
      ),
      workspace: workspace$.pipe(
        tap(v => this.saveObservable("workspace$", v)),
      ),
      system: system$.pipe(tap(v => this.saveObservable("system$", v))),
      changeSet: changeSet$.pipe(
        tap(v => this.saveObservable("changeSet$", v)),
      ),
      editSession: editSession$.pipe(
        tap(v => this.saveObservable("editSession$", v)),
      ),
      editMode: editMode$.pipe(tap(v => this.saveObservable("editMode$", v))),
      deploymentSchematicSelectNode: deploymentSchematicSelectNode$.pipe(
        tap(v => this.saveObservable("deploymentSchematicSelectNode$", v)),
      ),
      schematicSelectNode: schematicSelectNode$.pipe(
        tap(v => this.saveObservable("schematicSelectNode$", v)),
      ),
      panelTypeChanges: panelTypeChanges$.pipe(
        tap(v => {
          if (v) {
            let currentJson = sessionStorage.getItem("panelTypeChanges$");
            let key = `${v.applicationId}.${v.panelRef}`;
            if (currentJson) {
              let data: Record<string, PanelTypeChange> = JSON.parse(
                currentJson,
              );
              data[key] = v;
              this.saveObservable("panelTypeChanges$", data);
            } else {
              let data: Record<string, PanelTypeChange> = {};
              data[key] = v;
              this.saveObservable("panelTypeChanges$", data);
            }
          }
        }),
      ),
      schematicPanelKind: schematicPanelKind$.pipe(
        tap(v => {
          if (v) {
            let currentJson = sessionStorage.getItem("schematicPanelKind$");
            let key = `${v.applicationId}.${v.panelRef}`;
            if (currentJson) {
              let data: Record<string, SchematicPanelState> = JSON.parse(
                currentJson,
              );
              data[key] = v;
              this.saveObservable("schematicPanelKind$", data);
            } else {
              let data: Record<string, SchematicPanelState> = {};
              data[key] = v;
              this.saveObservable("schematicPanelKind$", data);
            }
          }
        }),
      ),
      attributePanelState: attributePanelState$.pipe(
        tap(v => {
          if (v) {
            let currentJson = sessionStorage.getItem("attributePanelState$");
            let key = `${v.applicationId}.${v.panelRef}`;
            if (currentJson) {
              let data: Record<string, AttributePanelState> = JSON.parse(
                currentJson,
              );
              data[key] = v;
              this.saveObservable("attributePanelState$", data);
            } else {
              let data: Record<string, AttributePanelState> = {};
              data[key] = v;
              this.saveObservable("attributePanelState$", data);
            }
          }
        }),
      ),
    };
  },
  methods: {
    saveObservable(key: string, value: any): void {
      if (this.saveAllowed) {
        sessionStorage.setItem(key, JSON.stringify(value));
      }
    },
    restoreObservable(key: string, observable: any): void {
      let json = sessionStorage.getItem(key);
      if (json) {
        let data = JSON.parse(json);
        observable.next(data);
      }
    },
    restoreState(): void {
      this.restoreObservable("user$", user$);
      this.restoreObservable("billingAccount$", billingAccount$);
      this.restoreObservable("organization$", organization$);
      this.restoreObservable("workspace$", workspace$);
      this.restoreObservable("system$", system$);
      this.restoreObservable("changeSet$", changeSet$);
      this.restoreObservable("editSession$", editSession$);
      this.restoreObservable("editMode$", editMode$);
      this.restoreObservable(
        "deploymentSchematicSelectNode$",
        deploymentSchematicSelectNode$,
      );
      this.restoreObservable("schematicSelectNode$", schematicSelectNode$);

      // Restore panel type selections
      let currentJson = sessionStorage.getItem("panelTypeChanges$");
      if (currentJson) {
        let data: Record<string, PanelTypeChange> = JSON.parse(currentJson);
        for (const panelTypeChange of Object.values(data)) {
          restorePanelTypeChanges$.next(panelTypeChange);
        }
      }

      // Restore Schematic Panel State
      let schematicJson = sessionStorage.getItem("schematicPanelKind$");
      if (schematicJson) {
        let data: Record<string, SchematicPanelState> = JSON.parse(
          schematicJson,
        );
        for (const schematicPanelState of Object.values(data)) {
          restoreSchematicPanelKind$.next(schematicPanelState);
        }
      }

      // Restore Attribute Panel State
      let attributeJson = sessionStorage.getItem("attributePanelState$");
      if (attributeJson) {
        let data: Record<string, AttributePanelState> = JSON.parse(
          attributeJson,
        );
        for (const attributePanelState of Object.values(data)) {
          restoreAttributePanelState$.next(attributePanelState);
        }
      }

      this.saveAllowed = true;
    },
  },
  mounted(): void {
    this.restoreState();
  },
});
</script>
