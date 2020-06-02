function generateEntity(
  workspaceId: string,
  number: number,
): Record<string, any> {
  return {
    name: `poop${number}`,
    displayName: "poopy pants",
    description: "really poopy",
    workspaceId: workspaceId,
    properties: {
      kubernetesObject: {
        kind: "your butt",
        apiVersion: "1.0",
      },
    },
    constraints: { kubernetesVersion: "V1_15" },
  };
}

function generateEntityFromVariables(
  workspaceId: string,
  number: number,
): Record<string, any> {
  return {
    name: `motherLoveBone${number}`,
    displayName: "Mother Love Bone",
    description: "Mother Love Bone",
    workspaceId: workspaceId,
    properties: {
      kubernetesObject: {
        apiVersion: "rr",
        kind: "rr",
        metadata: { name: "", labels: [] },
        spec: {
          replicas: "44",
          selector: { matchLabels: [] },
          template: {
            metadata: { name: "", labels: [] },
            spec: { containers: [] },
          },
        },
      },
    },
    constraints: {
      componentName: "",
      componentDisplayName: "",
      kubernetesVersion: "V1_15",
    },
  };
}

describe("kubernetesDeploymentEntity", () => {
  beforeEach(() => {
    cy.createUserBobo();
    cy.loginBobo().as("profile");
  });

  describe("create", () => {
    beforeEach(() => {
      cy.get("@profile").then((profile: Record<string, any>) => {
        cy.task("db:deleteByTypeName", {
          typeName: "kubernetes_deployment_entity",
          billingAccountId: profile["billingAccount"]["id"],
        });
      });
    });

    it("creates the entity", () => {
      cy.get("@profile").then((profile: Record<string, any>) => {
        cy.graphqlMutation({
          typeName: "kubernetesDeploymentEntity",
          queryArgs: {
            methodName: "create",
          },
          variables: generateEntity(profile["workspaceDefault"]["id"], 0),
        }).then((results: Record<string, any>) => {
          expect(results).to.have.property("data");
          expect(results["data"]).to.have.property(
            "kubernetesDeploymentEntityCreate",
          );
          const responseData =
            results["data"]["kubernetesDeploymentEntityCreate"];
          expect(responseData).to.have.property("item");
          expect(responseData["item"]).to.include({
            name: "poop0",
          });
          expect(responseData["item"]).to.include({
            displayName: "poopy pants",
          });
          expect(responseData["item"]).to.include({
            description: "really poopy",
          });
          expect(responseData["item"]).to.have.property("properties");
          expect(responseData["item"]["properties"]).to.have.property(
            "kubernetesObject",
          );
          expect(
            responseData["item"]["properties"]["kubernetesObject"],
          ).to.include({
            kind: "your butt",
            apiVersion: "1.0",
          });
        });
      });
    });

    it("empty changeSetId does not hang forever", () => {
      cy.get("@profile").then((profile: Record<string, any>) => {
        cy.graphqlMutation({
          typeName: "kubernetesDeploymentEntity",
          queryArgs: {
            methodName: "create",
          },
          variables: {
            changeSetId: "",
            ...generateEntityFromVariables(
              profile["workspaceDefault"]["id"],
              0,
            ),
          },
        }).then((results: Record<string, any>) => {
          expect(results).to.have.property("data");
          expect(results["data"]).to.have.property(
            "kubernetesDeploymentEntityCreate",
          );
          const responseData =
            results["data"]["kubernetesDeploymentEntityCreate"];
          expect(responseData).to.have.property("item");
          expect(responseData["item"]).to.include({
            name: "motherLoveBone0",
          });
          expect(responseData["item"]).to.include({
            displayName: "Mother Love Bone",
          });
          expect(responseData["item"]).to.include({
            description: "Mother Love Bone",
          });
          expect(responseData["item"]).to.have.property("properties");
          expect(responseData["item"]["properties"]).to.have.property(
            "kubernetesObject",
          );
        });
      });
    });
  });

  describe("get", () => {
    beforeEach(() => {
      cy.get("@profile").then((profile: Record<string, any>) => {
        cy.task("db:deleteByTypeName", {
          typeName: "kubernetes_deployment_entity",
          billingAccountId: profile["billingAccount"]["id"],
        });
      });
    });

    it("returns the entity", () => {
      cy.get("@profile").then((profile: Record<string, any>) => {
        cy.graphqlMutation({
          typeName: "kubernetesDeploymentEntity",
          queryArgs: {
            methodName: "create",
          },
          variables: generateEntity(profile["workspaceDefault"]["id"], 0),
        }).then((newItem: Record<string, any>) => {
          const itemId =
            newItem["data"]["kubernetesDeploymentEntityCreate"]["item"]["id"];

          cy.graphqlQuery({
            typeName: "kubernetesDeploymentEntity",
            queryArgs: {
              methodName: "get",
            },
            variables: {
              id: itemId,
            },
          }).then((results: Record<string, any>) => {
            expect(results).to.have.property("data");
            expect(results["data"]).to.have.property(
              "kubernetesDeploymentEntityGet",
            );
            const responseData =
              results["data"]["kubernetesDeploymentEntityGet"];
            expect(responseData).to.have.property("item");
          });
        });
      });
    });
  });

  describe("list", () => {
    beforeEach(() => {
      cy.get("@profile").then((profile: Record<string, any>) => {
        cy.task("db:deleteByTypeName", {
          typeName: "kubernetes_deployment_entity",
          billingAccountId: profile["billingAccount"]["id"],
        });
      });
    });

    it("is empty without any items", () => {
      cy.graphqlQuery({
        typeName: "kubernetesDeploymentEntity",
        queryArgs: {
          methodName: "list",
        },
      }).then((results: Record<string, any>) => {
        expect(results).to.have.property("data");
        expect(results["data"]).to.have.property(
          "kubernetesDeploymentEntityList",
        );
        const responseData = results["data"]["kubernetesDeploymentEntityList"];
        expect(responseData).to.have.property("items");
        expect(responseData["items"]).to.be.an("array").lengthOf(0);
        expect(responseData["nextPageToken"]).to.be.string("");
        expect(responseData["totalCount"]).to.be.string("0");
      });
    });

    it("returns all items when the list is small", () => {
      cy.get("@profile").then((profile: Record<string, any>) => {
        for (let count = 0; count < 5; count++) {
          cy.graphqlMutation({
            typeName: "kubernetesDeploymentEntity",
            queryArgs: {
              methodName: "create",
            },
            variables: generateEntity(profile["workspaceDefault"]["id"], count),
          });
        }
      });

      cy.graphqlQuery({
        typeName: "kubernetesDeploymentEntity",
        queryArgs: {
          methodName: "list",
        },
      }).then((results: Record<string, any>) => {
        expect(results).to.have.property("data");
        expect(results["data"]).to.have.property(
          "kubernetesDeploymentEntityList",
        );
        const responseData = results["data"]["kubernetesDeploymentEntityList"];
        expect(responseData).to.have.property("items");
        expect(responseData["items"]).to.be.an("array").lengthOf(5);
        expect(responseData["nextPageToken"]).to.be.string("");
        expect(responseData["totalCount"]).to.be.string("5");
      });
    });

    it("returns only the page size number of items when the list is large", () => {
      cy.get("@profile").then((profile: Record<string, any>) => {
        for (let count = 0; count < 50; count++) {
          cy.graphqlMutation({
            typeName: "kubernetesDeploymentEntity",
            queryArgs: {
              methodName: "create",
            },
            variables: generateEntity(profile["workspaceDefault"]["id"], count),
          });
        }
      });

      cy.graphqlQuery({
        typeName: "kubernetesDeploymentEntity",
        queryArgs: {
          methodName: "list",
        },
        variables: {
          pageSize: "10",
          orderBy: "name",
        },
      }).then((results: Record<string, any>) => {
        expect(results).to.have.property("data");
        expect(results["data"]).to.have.property(
          "kubernetesDeploymentEntityList",
        );
        const responseData = results["data"]["kubernetesDeploymentEntityList"];
        expect(responseData).to.have.property("items");
        expect(responseData["items"]).to.be.an("array").lengthOf(10);
        expect(responseData["nextPageToken"]).to.be.string;
        expect(responseData["totalCount"]).to.be.string("50");
        expect(responseData["items"][9]["name"]).to.be.string("poop17");
      });
    });
  });
});
