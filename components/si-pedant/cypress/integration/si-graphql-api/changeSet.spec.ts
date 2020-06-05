import { generateEntity as generateKubernetesDeploymentEntity } from "../../generators/kubernetesDeploymentEntity";

describe("changeSet", () => {
  beforeEach(() => {
    cy.logout();
    cy.task("db:deleteBoboCorp");
    cy.createUserBobo();
    cy.loginBobo().as("profile");
  });

  describe("create", () => {
    it("makes a new changeSet", () => {
      cy.get("@profile").then((profile: Record<string, any>) => {
        cy.graphqlMutation({
          typeName: "changeSet",
          queryArgs: {
            methodName: "create",
          },
          variables: {
            name: "gojira",
            displayName: "Gojira Lives!",
            workspaceId: profile["workspaceDefault"]["id"],
            createdByUserId: profile["user"]["id"],
          },
        }).should((rawResult: Record<string, any>) => {
          expect(rawResult).to.have.property("data");
          expect(rawResult["data"]).to.have.property("changeSetCreate");
          expect(rawResult["data"]["changeSetCreate"]).to.have.property("item");
          const item = rawResult["data"]["changeSetCreate"]["item"];
          expect(item).to.have.property("id").and.to.be.string;
          expect(item).to.have.property("name").and.to.be.eq("gojira");
          expect(item)
            .to.have.property("displayName")
            .and.to.be.eq("Gojira Lives!");
        });
      });
    });
  });

  describe("get", () => {
    beforeEach(() => {
      cy.get("@profile")
        .then((profile: Record<string, any>) => {
          cy.graphqlMutation({
            typeName: "changeSet",
            queryArgs: {
              methodName: "create",
            },
            variables: {
              name: "gojira",
              displayName: "Gojira Lives!",
              workspaceId: profile["workspaceDefault"]["id"],
              createdByUserId: profile["user"]["id"],
            },
          });
        })
        .as("newItem");
    });

    it("returns the item", () => {
      cy.get("@newItem").then((rawResult: Record<string, any>) => {
        const itemId = rawResult["data"]["changeSetCreate"]["item"]["id"];
        cy.graphqlQuery({
          typeName: "changeSet",
          queryArgs: {
            methodName: "get",
          },
          variables: {
            id: itemId,
          },
        }).should((results: Record<string, any>) => {
          expect(results).to.have.property("data");
          expect(results["data"]).to.have.property("changeSetGet");
          const responseData = results["data"]["changeSetGet"];
          expect(responseData).to.have.property("item");
          const item = responseData["item"];
          expect(item).to.have.property("id").and.to.be.string;
          expect(item).to.have.property("name").and.to.be.string("gojira");
          expect(item)
            .to.have.property("displayName")
            .and.to.be.string("Gojira Lives!");
        });
      });
    });
  });

  describe("entries", () => {
    beforeEach(() => {
      cy.get("@profile")
        .then((profile: Record<string, any>) => {
          cy.graphqlMutation({
            typeName: "changeSet",
            queryArgs: {
              methodName: "create",
            },
            variables: {
              name: "gojira",
              displayName: "Gojira Lives!",
              workspaceId: profile["workspaceDefault"]["id"],
              createdByUserId: profile["user"]["id"],
            },
          });
        })
        .as("changeSet");
    });

    describe("add an entry by creating an entity with the changeSetId", () => {
      it("creates an entity and it appears in the changeSetEntries association", () => {
        cy.get("@changeSet").then((changeSet: Record<string, any>) => {
          const changeSetId =
            changeSet["data"]["changeSetCreate"]["item"]["id"];
          const kde = generateKubernetesDeploymentEntity(
            changeSet["data"]["changeSetCreate"]["item"]["siProperties"][
              "workspaceId"
            ],
            0,
          );
          cy.graphqlMutation({
            typeName: "kubernetesDeploymentEntity",
            queryArgs: { methodName: "create" },
            variables: {
              changeSetId,
              ...kde,
            },
          })
            .should((changeSetEntry: Record<string, any>) => {
              expect(changeSetEntry)
                .to.have.property("data")
                .and.property("kubernetesDeploymentEntityCreate")
                .and.property("item");
              const item =
                changeSetEntry["data"]["kubernetesDeploymentEntityCreate"][
                  "item"
                ];
              expect(item)
                .to.have.property("id")
                .and.to.match(
                  new RegExp(`^${changeSetId}:0`),
                  "ID starts with changeSetId and changeSetEntryCount of 0",
                );
            })
            .then((changeSetEntry: Record<string, any>) => {
              const kdeItem =
                changeSetEntry["data"]["kubernetesDeploymentEntityCreate"][
                  "item"
                ];
              cy.graphqlQuery({
                typeName: "changeSet",
                queryArgs: {
                  methodName: "get",
                  associations: {
                    changeSet: ["changeSetEntries"],
                  },
                },
                variables: {
                  id: changeSetId,
                },
              }).should((changeSetEntryAssoc: Record<string, any>) => {
                expect(changeSetEntryAssoc)
                  .to.have.property("data")
                  .and.property("changeSetGet")
                  .and.property("item");
                const item =
                  changeSetEntryAssoc["data"]["changeSetGet"]["item"];
                expect(item)
                  .to.have.property("associations")
                  .and.property("changeSetEntries");
                const changeSetEntries =
                  item["associations"]["changeSetEntries"];
                expect(changeSetEntries)
                  .to.have.property("items")
                  .and.length(1);
                expect(changeSetEntries["items"][0]).to.include({
                  id: kdeItem.id,
                });
              });
            });
        });
      });

      it("shows many entries in the changeSetEntries", () => {
        cy.get("@changeSet").then((changeSet: Record<string, any>) => {
          const changeSetId =
            changeSet["data"]["changeSetCreate"]["item"]["id"];
          for (let count = 0; count < 5; count++) {
            const kde = generateKubernetesDeploymentEntity(
              changeSet["data"]["changeSetCreate"]["item"]["siProperties"][
                "workspaceId"
              ],
              count,
            );
            cy.graphqlMutation({
              typeName: "kubernetesDeploymentEntity",
              queryArgs: { methodName: "create" },
              variables: {
                changeSetId,
                ...kde,
              },
            });
          }
          cy.graphqlQuery({
            typeName: "changeSet",
            queryArgs: {
              methodName: "get",
              associations: {
                changeSet: ["changeSetEntries"],
              },
            },
            variables: {
              id: changeSetId,
            },
          }).should((changeSetEntryAssoc: Record<string, any>) => {
            expect(changeSetEntryAssoc)
              .to.have.property("data")
              .and.property("changeSetGet")
              .and.property("item");
            const item = changeSetEntryAssoc["data"]["changeSetGet"]["item"];
            expect(item)
              .to.have.property("associations")
              .and.property("changeSetEntries");
            const changeSetEntries = item["associations"]["changeSetEntries"];
            expect(changeSetEntries).to.have.property("items").and.length(5);
          });
        });
      });
    });

    describe("add an entry by deleting an entity with the changeSetId", () => {
      it("deletes an entity and it appears in the changeSetEntries association", () => {
        cy.get("@changeSet").then((changeSet: Record<string, any>) => {
          const changeSetId =
            changeSet["data"]["changeSetCreate"]["item"]["id"];
          const kde = generateKubernetesDeploymentEntity(
            changeSet["data"]["changeSetCreate"]["item"]["siProperties"][
              "workspaceId"
            ],
            0,
          );
          cy.graphqlMutation({
            typeName: "kubernetesDeploymentEntity",
            queryArgs: { methodName: "create" },
            variables: {
              changeSetId,
              ...kde,
            },
          }).as("createdEntity");
          cy.get("@createdEntity").then(
            (changeSetEntry: Record<string, any>) => {
              const item =
                changeSetEntry["data"]["kubernetesDeploymentEntityCreate"][
                  "item"
                ];

              cy.graphqlMutation({
                typeName: "kubernetesDeploymentEntity",
                queryArgs: {
                  methodName: "delete",
                },
                variables: {
                  id: item["id"],
                  changeSetId,
                },
              }).as("deletedEntity");
            },
          );
          cy.get("@deletedEntity").then(
            (deletedEntity: Record<string, any>) => {
              const kdeItem =
                deletedEntity["data"]["kubernetesDeploymentEntityDelete"][
                  "item"
                ];
              cy.graphqlQuery({
                typeName: "changeSet",
                queryArgs: {
                  methodName: "get",
                  associations: {
                    changeSet: ["changeSetEntries"],
                  },
                },
                variables: {
                  id: changeSetId,
                },
              }).should((changeSetEntryAssoc: Record<string, any>) => {
                expect(changeSetEntryAssoc)
                  .to.have.property("data")
                  .and.property("changeSetGet")
                  .and.property("item");
                const item =
                  changeSetEntryAssoc["data"]["changeSetGet"]["item"];
                expect(item)
                  .to.have.property("associations")
                  .and.property("changeSetEntries");
                const changeSetEntries =
                  item["associations"]["changeSetEntries"];
                expect(changeSetEntries)
                  .to.have.property("items")
                  .and.length(2);
                expect(changeSetEntries["items"][1]).to.include({
                  id: kdeItem.id,
                });
              });
            },
          );
        });
      });
    });

    // We should be able to Remove entries from a changeSet by
    // calling some method and passing the id of our changeSet
    // entity.

    // We should be able to change the visibility from public
    // to private, etc.

    // We should be able to abandon a changeset, which simply
    // marks it as abandoned.

    // We should be able to delete a changeset, which removes
    // the changeSet and any related entries.

    // We should be able to submit a changeset for approval,
    // which creates an approval process that allows others
    // to comment on the changeSet, and either Execute, Abandon,
    // or Delete it.

    // We should be able to Execute a changeSet, which walks
    // the list of entries and applies them to the true state
    // of the system.
    describe("execute", () => {
      beforeEach(() => {
        cy.get("@changeSet").then((changeSet: Record<string, any>) => {
          const changeSetId =
            changeSet["data"]["changeSetCreate"]["item"]["id"];
          for (let count = 0; count < 5; count++) {
            const kde = generateKubernetesDeploymentEntity(
              changeSet["data"]["changeSetCreate"]["item"]["siProperties"][
                "workspaceId"
              ],
              count,
            );
            cy.graphqlMutation({
              typeName: "kubernetesDeploymentEntity",
              queryArgs: { methodName: "create" },
              variables: {
                changeSetId,
                ...kde,
              },
            }).as(`createdEntity${count}`);
          }
        });
      });

      it("executes multiple change set create entries in order", () => {
        cy.get("@changeSet").then((changeSet: Record<string, any>) => {
          const changeSetId =
            changeSet["data"]["changeSetCreate"]["item"]["id"];

          cy.graphqlMutation({
            typeName: "changeSet",
            queryArgs: {
              methodName: "execute",
              associations: {
                changeSet: ["changeSetEntries"],
              },
            },
            variables: {
              id: changeSetId,
            },
          }).should((result: Record<string, any>) => {
            expect(result)
              .to.have.property("data")
              .and.property("changeSetExecute")
              .and.property("item");
            const item = result["data"]["changeSetExecute"]["item"];
            expect(item).to.have.property("id").and.eq(changeSetId);
            expect(item)
              .to.have.property("status")
              .and.eq("EXECUTING", "status should be EXECUTING");
            expect(item)
              .to.have.property("associations")
              .and.property("changeSetEntries")
              .and.property("items")
              .lengthOf(5);
            for (const entry of item["associations"]["changeSetEntries"][
              "items"
            ]) {
              expect(entry).to.have.property("id");
              const extractIdExp = /^change_set:.+:\d+:(.+)$/;
              const extractResult = extractIdExp.exec(entry["id"]);
              let realId: string;
              if (extractResult) {
                realId = extractResult[1];
              } else {
                throw new Error("cannot extract ID from entity");
              }
              expect(realId)
                .to.be.a("string")
                .and.to.match(/^kubernetes_deployment_entity:/);
              cy.graphqlQuery({
                typeName: "kubernetesDeploymentEntity",
                queryArgs: { methodName: "get" },
                variables: { id: realId },
              }).should((realEntityResult: Record<string, any>) => {
                const realEntity =
                  realEntityResult["data"]["kubernetesDeploymentEntityGet"][
                    "item"
                  ];
                expect(realEntity).to.have.property("id").and.to.eq(realId);
              });
            }
          });

          cy.graphqlQuery({
            typeName: "changeSet",
            queryArgs: { methodName: "get" },
            variables: { id: changeSetId },
          }).should((changeSetResult: Record<string, any>) => {
            const item = changeSetResult["data"]["changeSetGet"]["item"];
            expect(item).to.have.property("status").and.eq("CLOSED");
          });
        });
      });

      it("executes multiple entries in order with a delete", () => {
        cy.get("@createdEntity0").then((createdEntity: Record<string, any>) => {
          const item =
            createdEntity["data"]["kubernetesDeploymentEntityCreate"]["item"];

          cy.graphqlMutation({
            typeName: "kubernetesDeploymentEntity",
            queryArgs: {
              methodName: "delete",
            },
            variables: {
              id: item["id"],
              changeSetId: item["siStorable"]["changeSetId"],
            },
          }).as("deletedEntity0");
        });

        cy.get("@changeSet").then((changeSet: Record<string, any>) => {
          const changeSetId =
            changeSet["data"]["changeSetCreate"]["item"]["id"];

          cy.graphqlMutation({
            typeName: "changeSet",
            queryArgs: {
              methodName: "execute",
              associations: {
                changeSet: ["changeSetEntries"],
              },
            },
            variables: {
              id: changeSetId,
            },
          }).should((result: Record<string, any>) => {
            expect(result)
              .to.have.property("data")
              .and.property("changeSetExecute")
              .and.property("item");
            const item = result["data"]["changeSetExecute"]["item"];
            expect(item).to.have.property("id").and.eq(changeSetId);
            expect(item)
              .to.have.property("status")
              .and.eq("EXECUTING", "status should be EXECUTING");
            expect(item)
              .to.have.property("associations")
              .and.property("changeSetEntries")
              .and.property("items")
              .lengthOf(6);
            for (const entry of item["associations"]["changeSetEntries"][
              "items"
            ]) {
              expect(entry).to.have.property("id");
              const extractIdExp = /^change_set:.+:\d+:(.+)$/;
              const extractResult = extractIdExp.exec(entry["id"]);
              let realId: string;
              if (extractResult) {
                realId = extractResult[1];
              } else {
                throw new Error("cannot extract ID from entity");
              }
              expect(realId)
                .to.be.a("string")
                .and.to.match(/^kubernetes_deployment_entity:/);
              cy.graphqlQuery({
                typeName: "kubernetesDeploymentEntity",
                queryArgs: { methodName: "get" },
                variables: { id: realId },
              }).should((realEntityResult: Record<string, any>) => {
                const realEntity =
                  realEntityResult["data"]["kubernetesDeploymentEntityGet"][
                    "item"
                  ];
                expect(realEntity).to.have.property("id").and.to.eq(realId);
              });
            }
          });

          cy.graphqlQuery({
            typeName: "changeSet",
            queryArgs: { methodName: "get" },
            variables: { id: changeSetId },
          }).should((changeSetResult: Record<string, any>) => {
            const item = changeSetResult["data"]["changeSetGet"]["item"];
            expect(item).to.have.property("status").and.eq("CLOSED");
          });

          cy.get("@deletedEntity0").then(
            (deletedEntity: Record<string, any>) => {
              const item =
                deletedEntity["data"]["kubernetesDeploymentEntityDelete"][
                  "item"
                ];

              cy.graphqlQuery({
                typeName: "kubernetesDeploymentEntity",
                queryArgs: { methodName: "get" },
                variables: {
                  id: item["siStorable"]["itemId"],
                },
              }).should((finalEntity: Record<string, any>) => {
                const finalItem =
                  finalEntity["data"]["kubernetesDeploymentEntityGet"]["item"];
                expect(finalItem)
                  .to.have.property("siStorable")
                  .and.property("deleted")
                  .to.be.eq(true);
              });
            },
          );
        });
      });

      it("executes multiple entries in order with an update", () => {
        cy.get("@createdEntity0").then((createdEntity: Record<string, any>) => {
          const item =
            createdEntity["data"]["kubernetesDeploymentEntityCreate"]["item"];

          cy.graphqlMutation({
            typeName: "kubernetesDeploymentEntity",
            queryArgs: {
              methodName: "update",
            },
            variables: {
              id: item["id"],
              changeSetId: item["siStorable"]["changeSetId"],
              update: {
                name: item["name"],
                description: item["description"],
                properties: {
                  kubernetesObject: {
                    apiVersion:
                      item["properties"]["kubernetesObject"]["apiVersion"],
                    kind: item["properties"]["kubernetesObject"]["kind"],
                    spec: {
                      replicas: 15,
                    },
                  },
                },
              },
            },
          }).as("updatedEntity0");
        });

        cy.get("@changeSet").then((changeSet: Record<string, any>) => {
          const changeSetId =
            changeSet["data"]["changeSetCreate"]["item"]["id"];

          cy.graphqlMutation({
            typeName: "changeSet",
            queryArgs: {
              methodName: "execute",
              associations: {
                changeSet: ["changeSetEntries"],
              },
            },
            variables: {
              id: changeSetId,
            },
          }).should((result: Record<string, any>) => {
            expect(result)
              .to.have.property("data")
              .and.property("changeSetExecute")
              .and.property("item");
            const item = result["data"]["changeSetExecute"]["item"];
            expect(item).to.have.property("id").and.eq(changeSetId);
            expect(item)
              .to.have.property("status")
              .and.eq("EXECUTING", "status should be EXECUTING");
            expect(item)
              .to.have.property("associations")
              .and.property("changeSetEntries")
              .and.property("items")
              .lengthOf(6);
            for (const entry of item["associations"]["changeSetEntries"][
              "items"
            ]) {
              expect(entry).to.have.property("id");
              const extractIdExp = /^change_set:.+:\d+:(.+)$/;
              const extractResult = extractIdExp.exec(entry["id"]);
              let realId: string;
              if (extractResult) {
                realId = extractResult[1];
              } else {
                throw new Error("cannot extract ID from entity");
              }
              expect(realId)
                .to.be.a("string")
                .and.to.match(/^kubernetes_deployment_entity:/);
              cy.graphqlQuery({
                typeName: "kubernetesDeploymentEntity",
                queryArgs: { methodName: "get" },
                variables: { id: realId },
              }).should((realEntityResult: Record<string, any>) => {
                const realEntity =
                  realEntityResult["data"]["kubernetesDeploymentEntityGet"][
                    "item"
                  ];
                expect(realEntity).to.have.property("id").and.to.eq(realId);
              });
            }
          });

          cy.graphqlQuery({
            typeName: "changeSet",
            queryArgs: { methodName: "get" },
            variables: { id: changeSetId },
          }).should((changeSetResult: Record<string, any>) => {
            const item = changeSetResult["data"]["changeSetGet"]["item"];
            expect(item).to.have.property("status").and.eq("CLOSED");
          });

          cy.get("@updatedEntity0").then(
            (updatedEntity: Record<string, any>) => {
              const item =
                updatedEntity["data"]["kubernetesDeploymentEntityUpdate"][
                  "item"
                ];

              cy.graphqlQuery({
                typeName: "kubernetesDeploymentEntity",
                queryArgs: { methodName: "get" },
                variables: {
                  id: item["siStorable"]["itemId"],
                },
              }).should((finalEntity: Record<string, any>) => {
                const finalItem =
                  finalEntity["data"]["kubernetesDeploymentEntityGet"]["item"];
                expect(finalItem)
                  .to.have.property("siStorable")
                  .and.property("deleted")
                  .to.be.eq(false);

                expect(finalItem)
                  .to.have.property("properties")
                  .and.property("kubernetesObject")
                  .and.property("spec")
                  .and.property("replicas")
                  .to.be.eq(15);
              });
            },
          );
        });
      });

      it("executes multiple entries in order with an action", () => {
        cy.get("@createdEntity0").then((createdEntity: Record<string, any>) => {
          const item =
            createdEntity["data"]["kubernetesDeploymentEntityCreate"]["item"];

          cy.graphqlMutation({
            typeName: "kubernetesDeploymentEntity",
            queryArgs: {
              methodName: "sync",
            },
            variables: {
              id: item["id"],
            },
          }).as("syncEntityEvent");
        });

        cy.get("@changeSet").then((changeSet: Record<string, any>) => {
          const changeSetId =
            changeSet["data"]["changeSetCreate"]["item"]["id"];

          cy.graphqlMutation({
            typeName: "changeSet",
            queryArgs: {
              methodName: "execute",
              associations: {
                changeSet: ["changeSetEntries"],
              },
            },
            variables: {
              id: changeSetId,
            },
          }).should((result: Record<string, any>) => {
            expect(result)
              .to.have.property("data")
              .and.property("changeSetExecute")
              .and.property("item");
            const item = result["data"]["changeSetExecute"]["item"];
            expect(item).to.have.property("id").and.eq(changeSetId);
            expect(item)
              .to.have.property("status")
              .and.eq("EXECUTING", "status should be EXECUTING");
            expect(item)
              .to.have.property("associations")
              .and.property("changeSetEntries")
              .and.property("items")
              .lengthOf(6);
            const entry = item["associations"]["changeSetEntries"]["items"][5];

            expect(entry).to.have.property("id");
            const extractIdExp = /^change_set:.+:\d+:(.+)$/;
            const extractResult = extractIdExp.exec(entry["id"]);
            let realId: string;
            if (extractResult) {
              realId = extractResult[1];
            } else {
              throw new Error("cannot extract ID from entity");
            }
            expect(realId)
              .to.be.a("string")
              .and.to.match(/^kubernetes_deployment_entity_event:/);
          });

          cy.graphqlQuery({
            typeName: "changeSet",
            queryArgs: { methodName: "get" },
            variables: { id: changeSetId },
          }).should((changeSetResult: Record<string, any>) => {
            const item = changeSetResult["data"]["changeSetGet"]["item"];
            expect(item).to.have.property("status").and.eq("CLOSED");
          });
        });
      });
    });
  });
});
