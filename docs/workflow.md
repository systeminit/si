- Feature Name: Workflow
- Start Date: 2022-08-08

# Summary

[summary]: #summary

# Motivation

[motivation]: #motivation

# Guide-level explanation

[guide-level-explanation]: #guide-level-explanation

# Reference-level explanation

[reference-level-explanation]: #reference-level-explanation

# Drawbacks

[drawbacks]: #drawbacks

# Rationale and alternatives

[rationale-and-alternatives]: #rationale-and-alternatives

# Prior art

[prior-art]: #prior-art

## Workflow engines

### Temporal

<https://docs.temporal.io/workflows>

### Apache Airflow

<https://airflow.apache.org/docs/apache-airflow/stable/concepts/overview.html>
<https://airflow.apache.org/docs/apache-airflow/stable/concepts/dags.html>
<https://airflow.apache.org/docs/apache-airflow/stable/concepts/tasks.html>

### Brigade

<https://docs.brigade.sh/topics/scripting/guide/>

### Argo workflows

<https://argoproj.github.io/argo-workflows/walk-through/parameters/>
<https://argoproj.github.io/argo-workflows/walk-through/steps/>
<https://argoproj.github.io/argo-workflows/walk-through/dag/>
<https://argoproj.github.io/argo-workflows/walk-through/output-parameters/>
<https://argoproj.github.io/argo-workflows/walk-through/conditionals/>

### Azkaban

<https://azkaban.readthedocs.io/en/latest/createFlows.html>

### CloudSlang

<https://cloudslang-docs.readthedocs.io/en/latest/cloudslang/cloudslang_examples.html>

### Couler

<https://couler-proj.github.io/couler/examples/>

### Dagu

<https://github.com/yohamta/dagu>

## Continuous integration solutions

### GitHub Actions

<https://docs.github.com/en/actions/learn-github-actions/understanding-github-actions>

### CircleCI

<https://circleci.com/docs/workflows>

### Cirrus CI

<https://cirrus-ci.org/guide/writing-tasks/>
<https://cirrus-ci.org/guide/programming-tasks/>

### Travis Cirrus

<https://docs.travis-ci.com/user/job-lifecycle/>
<https://docs.travis-ci.com/user/build-stages/>
<https://docs.travis-ci.com/user/conditional-builds-stages-jobs/>

## Specifications

### Workflow Description Language (WDL)

<https://support.terra.bio/hc/en-us/articles/360037118252-Base-structure>
<https://support.terra.bio/hc/en-us/articles/360037486731-Add-plumbing>

### Common Workflow Language (CWL)

> CWL workflows describe each step with explict inputs and outputs. Workflow
> steps in CWL are not necessarily run in the order they are listed, instead the
> **order is determined by the dependencies between steps**. Workflow steps
> which do not depend on one another may run in parallel.
> ([source](https://www.commonwl.org/features/#parallelization-and-scale-with-cwl))

<https://www.commonwl.org/user_guide/21-1st-workflow/index.html>

## Notable user experiences

- [Kestra](https://kestra.io/): _"Edit, run, and monitor in real time directly
  on the all-in-one web interface"_
- [n8n](https://kestra.io/): Free and open node-based workflow automation tool.
  Easily automate tasks across different services. _"Code when you need it, UI
  when you don't"_

## Misc

- [awesome-workflow-engines](https://github.com/meirwah/awesome-workflow-engines):
  A curated list of awesome open source workflow engines
- [Common Workflow Language: Existing Workflow systems](https://github.com/common-workflow-language/common-workflow-language/wiki/Existing-Workflow-systems):
  Curated list by CWL project of other systems

# Unresolved questions

[unresolved-questions]: #unresolved-questions

- Should tasks be generic/re-sharable or simply contained in a simpler workflow
  which itself can be reused?

# Future possibilities

[future-possibilities]: #future-possibilities
