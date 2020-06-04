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

    describe("create", () => {
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

      it("shows many items in the changeSetEntries", () => {
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

    describe.only("apply", () => {
      it("does nothing on an empty changeSet", () => {
        cy.get("@changeSet").then((changeSet: Record<string, any>) => {
          const changeSetId =
            changeSet["data"]["changeSetCreate"]["item"]["id"];
        });
      });
    });
  });
});
