import { uniqueNamesGenerator, adjectives, colors, animals } from "unique-names-generator";

interface NodeBuilder {
    name: string;
    menuItem: string;
    menu: string;
    typeName: string;
    row: boolean;
    secret?: string;
    properties: {
        path: string[];
        value: string;
        select?: boolean;
    }[];
    configures: {
        typeName: string;
        name: string;
    }[];
}

interface SiContext {
    billingAccountName: string;
    awsAccessKeyId: string;
    awsSecretKeyId: string;
    awsAccessKeyCredentialName: string;
    dockerHubUsername: string;
    dockerHubPassword: string;
    dockerHubCredentialName: string;
    applicationName: string;
    changeSetName: string;
    nodeCounter: number;
    nodeSizeX: number;
    nodeSizeY: number;
    newRowPositionX: number;
    startingPositionX: number;
    startingPositionY: number;
    nodes: NodeBuilder[];
}


context("system initiative", () => {
    before(() => {
        let context: SiContext = {
            billingAccountName: uniqueNamesGenerator({ dictionaries: [adjectives, colors, animals] }),
            awsAccessKeyId: Cypress.env("aws_access_key_id"),
            awsSecretKeyId: Cypress.env("aws_secret_access_key"),
            awsAccessKeyCredentialName: "awsProd",
            dockerHubUsername: Cypress.env("docker_hub_username"),
            dockerHubPassword: Cypress.env("docker_hub_password"),
            dockerHubCredentialName: "systeminit",
            applicationName: uniqueNamesGenerator({ dictionaries: [adjectives, colors, animals] }),
            changeSetName: uniqueNamesGenerator({ dictionaries: [adjectives, colors, animals] }),
            nodeCounter: 0,
            nodeSizeX: 138,
            nodeSizeY: 98,
            startingPositionX: 350,
            newRowPositionX: 350,
            startingPositionY: 325,
            nodes: [
                {
                    name: "awsProd",
                    menu: "aws",
                    menuItem: "access-key",
                    typeName: "awsAccessKeyCredential",
                    secret: "awsProd",
                    properties: [],
                    configures: [
                    ],
                    row: false,
                },
                {
                    name: "si-dockerhub",
                    menu: "docker",
                    menuItem: "credential",
                    typeName: "dockerHubCredential",
                    secret: "systeminit",
                    properties: [],
                    configures: [
                        {
                            typeName: "dockerImage",
                            name: "si-sdf",
                        },
                        {
                            typeName: "dockerImage",
                            name: "si-veritech",
                        },
                    ],
                    row: true,
                },
                {
                    name: "awsRegion",
                    menu: "aws",
                    menuItem: "region",
                    typeName: "aws",
                    properties: [
                        {
                            path: ["region"],
                            value: "us-east-2",
                            select: true,
                        }
                    ],
                    configures: [
                        {
                            typeName: "awsEks",
                            name: "k8s",
                        }
                    ],
                    row: true,
                },
                {
                    name: "k8s",
                    menu: "aws",
                    menuItem: "eks--cluster",
                    typeName: "awsEks",
                    properties: [
                        {
                            path: ["clusterName"],
                            value: Cypress.env("k8s_cluster_name"),
                        }
                    ],
                    configures: [],
                    row: false,
                },
                {
                    name: "si-sdf",
                    menu: "application",
                    menuItem: "service",
                    typeName: "service",
                    properties: [],
                    configures: [
                        {
                            typeName: "dockerImage",
                            name: "si-sdf",
                        },
                        {
                            typeName: "dockerImage",
                            name: "si-veritech",
                        },
                    ],
                    row: false,
                },
                {
                    name: "si-sdf",
                    menu: "docker",
                    menuItem: "image",
                    typeName: "dockerImage",
                    properties: [{
                        path: ["image"],
                        value: "systeminit/si-sdf"
                    }],
                    configures: [],
                    row: false,
                },
                {
                    name: "si-veritech",
                    menu: "docker",
                    menuItem: "image",
                    typeName: "dockerImage",
                    properties: [{
                        path: ["image"],
                        value: "systeminit/si-veritech"
                    }],
                    configures: [],
                    row: false,
                },
            ]
        }
        cy.wrap(context).as("globalContext");
    });

    it('can deploy itself', function () {
        cy.viewport(1440, 1440);
        // @ts-ignore - we know that the context is an SiContext, thanks
        cy.get("@globalContext").then((ctx: SiContext) => {
            cy.log("Signup", { billingAccountName: ctx.billingAccountName, poop: ctx });
            cy.visit("/signup", { timeout: 10000 });
            cy.get("[data-cy=billingAccountName]").type(ctx.billingAccountName);
            cy.get("[data-cy=billingAccountDescription]").type("a");
            cy.get("[data-cy=userName]").type("a");
            cy.get("[data-cy=userEmail]").type("a");
            cy.get("[data-cy=userPassword]").type("a");
            cy.get("[data-cy=userPasswordAgain]").type("a");
            cy.get("[data-cy=create]").click();

            cy.log("Signin");
            cy.url().should("eq", `${Cypress.config().baseUrl}/signin`, { timeout: 20000 });
            cy.get("[data-cy=billingAccountName]").type(ctx.billingAccountName);
            cy.get("[data-cy=userEmail").type("a");
            cy.get("[data-cy=userPassword").type("a");
            cy.get("[data-cy=signInButton").click();

            cy.log("Add AWS Secret");
            cy.get("[data-cy=secret-nav-link]", { timeout: 20000 }).click();
            cy.get("[data-cy=new-secret-button]").click();
            cy.get("[data-cy=new-secret-form-secret-name]").type(ctx.awsAccessKeyCredentialName);
            cy.get("[data-cy=new-secret-form-kind]").select("awsAccessKey");
            cy.get("[data-cy=new-secret-form-access-key-id]").type(ctx.awsAccessKeyId);
            cy.get("[data-cy=new-secret-form-secret-access-key]").type(ctx.awsSecretKeyId);
            cy.get("[data-cy=new-secret-form-create-button]").click();

            cy.log("Add Docker Secret");
            cy.get("[data-cy=secret-nav-link]", { timeout: 20000 }).click();
            cy.get("[data-cy=new-secret-button]").click();
            cy.get("[data-cy=new-secret-form-secret-name]").type(ctx.dockerHubCredentialName);
            cy.get("[data-cy=new-secret-form-kind]").select("dockerHub");
            cy.get("[data-cy=new-secret-form-docker-hub-username]").type(ctx.dockerHubUsername);
            cy.get("[data-cy=new-secret-form-docker-hub-password]").type(ctx.dockerHubPassword);
            cy.get("[data-cy=new-secret-form-create-button]").click();

            cy.log("Create Application");
            cy.get("[data-cy=application-nav-link]").click();
            cy.get("[data-cy=new-application-button]").click();
            cy.get("[data-cy=new-application-form-application-name]").type(ctx.applicationName);
            cy.get("[data-cy=new-application-form-create-button]").click();

            cy.log("Enter edit mode");
            cy.get("[data-cy=application-details-mode-toggle]", { timeout: 30000 }).click({ timeout: 30000 });
            cy.get("[data-cy=new-change-set-form-name]").type(ctx.changeSetName);
            cy.get("[data-cy=new-change-set-form-create-button]").click();
        });

        // Create Nodes
        // @ts-ignore
        cy.get("@globalContext").then((ctx: SiContext) => {
            for (const nodeItem of ctx.nodes) {
                cy.log(`Creating ${nodeItem.menu} ${nodeItem.menuItem} node`, { nodeItem });
                cy.get("[data-cy=editor-schematic-node-add-button]").click();
                cy.get(`[data-cy=editor-schematic-node-add-menu-${nodeItem.menu}]`).click({ force: true });
                cy.get(`[data-cy=editor-schematic-node-add-menu-${nodeItem.menu}-${nodeItem.menuItem}]`).click({ force: true });
                cy.wait(2000);
                cy.get(`[data-cy=editor-schematic-panel-node-list-0]`).trigger("mousedown", { which: 1 }).wait(2000).trigger("mouseup");
                cy.get("[data-cy=editor-property-viewer-node-name-field]", { timeout: 5000 }).clear().type(nodeItem.name).trigger("blur").wait(1000);
                if (nodeItem.secret) {
                    cy.get("[data-cy=editor-property-viewer-secret-select]", { timeout: 5000 }).select(nodeItem.secret).trigger("blur").wait(1000);
                }
                for (const prop of nodeItem.properties) {
                    if (prop.select) {
                        cy.get(`[data-cy=editor-property-viewer-prop-${prop.path.join('-')}]`, { timeout: 5000 })
                            .select(prop.value)
                            .trigger("blur");
                    } else {
                        cy.get(`[data-cy=editor-property-viewer-prop-${prop.path.join('-')}]`, { timeout: 5000 })
                            .clear()
                            .type(prop.value)
                            .trigger("blur");
                    }
                }
                cy.get(`[data-cy=editor-schematic-panel-node-list-0]`).trigger("mousedown", { which: 1 }).wait(2000)
                    .trigger('mousemove', { clientX: ctx.startingPositionX, clientY: ctx.startingPositionY, }) //pageX: 50, pageY: 50, x: 50, y: 50 })
                    .trigger('mouseup', { clientX: ctx.startingPositionX, clientY: ctx.startingPositionY, });

                if (nodeItem.row) {
                    ctx.startingPositionX = ctx.newRowPositionX;
                    ctx.startingPositionY = ctx.startingPositionY + ctx.nodeSizeY + 20;
                } else {
                    ctx.startingPositionX = ctx.startingPositionX + ctx.nodeSizeX + 20;
                }
                ctx.nodeCounter++;
            }

            // Connection pass
            for (const nodeItem of ctx.nodes) {
                if (nodeItem.configures.length) {
                    for (const childItem of nodeItem.configures) {
                        cy.log(`Connecting ${nodeItem.typeName} ${nodeItem.name} to ${childItem.typeName} ${childItem.name}`);
                        cy.get(`[data-cy-name=editor-schematic-panel-node-list-${nodeItem.typeName}-${nodeItem.name}]`)
                            .trigger("mousedown", { which: 1 })
                            .trigger("mouseup")
                            .wait(1000);
                        cy.get(`[data-cy=editor-property-viewer-configures-select]`).select(`${childItem.typeName} ${childItem.name}`);
                        cy.get(`[data-cy=editor-property-viewer-configures-button]`).click()
                    }
                }
            }
        });

        cy.log("wtf");
    })
})