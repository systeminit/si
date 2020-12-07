// This file contains all the component data, in the order
// that it should be loaded.
//
// The are all loaded exclusively for their side-effects.

import "./components/si-cea/component";
import "./components/si-cea/entity";
import "./components/si-cea/entityEvent";

import "./components/si-data/data";

import "./components/si-account/billingAccount";
import "./components/si-account/capability";
import "./components/si-account/user";
import "./components/si-account/group";
import "./components/si-account/organization";
import "./components/si-account/workspace";
import "./components/si-account/integration";
import "./components/si-account/integrationService";
import "./components/si-account/integrationInstance";
import "./components/si-account/changeSet";
import "./components/si-account/editSession";
import "./components/si-account/item";
import "./components/si-account/eventLog";

import "./components/si-core/application";
import "./components/si-application/service";
import "./components/si-core/system";
import "./components/si-core/edge";
import "./components/si-core/resource";
import "./components/si-core/node";
import "./components/si-core/server";
import "./components/si-core/ubuntu";
import "./components/si-core/ami";
import "./components/si-core/ec2instance";
import "./components/si-docker/dockerHubCredential";
import "./components/si-aws/aws";
import "./components/si-aws/awsAccessKeyCredential";
import "./components/si-docker/dockerImage";

import "./components/si-kubernetes/base/container";
import "./components/si-kubernetes/base/metadata";
import "./components/si-kubernetes/base/podSpec";
import "./components/si-kubernetes/base/podTemplateSpec";
import "./components/si-kubernetes/base/loadBalancerStatus";
import "./components/si-kubernetes/base/selector";
import "./components/si-kubernetes/base/servicePort";

import "./components/si-kubernetes/base/serviceBackendPort";
import "./components/si-kubernetes/base/ingressServiceBackend";
import "./components/si-kubernetes/base/typedLocalObjectReference";
import "./components/si-kubernetes/base/ingressBackend";
import "./components/si-kubernetes/base/ingressSpec";

import "./components/si-kubernetes/base/httpIngressPath";
import "./components/si-kubernetes/base/httpIngressRuleValue";
import "./components/si-kubernetes/base/ingressRule";
import "./components/si-kubernetes/base/ingressTls";
import "./components/si-kubernetes/base/ingressStatus";
import "./components/si-kubernetes/entity/cluster";

import "./components/si-aws/awsIamJsonPolicy";
import "./components/si-aws/awsEks";

import "./components/si-helm/helmRepoCredential";
import "./components/si-helm/helmRepo";
import "./components/si-helm/helmChart";
import "./components/si-helm/helmRelease";

import "./components/si-kubernetes/entity/namespace";
import "./components/si-kubernetes/entity/secret";
import "./components/si-kubernetes/entity/deployment";
import "./components/si-kubernetes/entity/service";
import "./components/si-kubernetes/entity/ingress";
