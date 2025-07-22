async function main({
    thisComponent,
    components
}: Input): Promise < Output > {
    const vars = thisComponent.properties.domain;
    const namePrefix = vars["Name Prefix"];
    const sources = thisComponent.sources;
    const componentsToCreate: Array < Output["ops"]["create"][string] > = [];

    template.checkUniqueNamePrefix(namePrefix, components);

    // Component Definitions
    {%- for component in components %}
    const {{ component.variable_name }}: Output["ops"]["create"][string] = {
        kind: "{{ component.kind }}",
        attributes: {
            "/si/name": namePrefix + "{{ component.name }}",
            {%- for attr in component.attributes_pruned_and_sorted() %}
            {{ attr.dest_path | json }}:
            {%- match attr.value %}
              {%- when AttributeSource::Value with (val) %} {{ val | json(4) | indent(12) }},
              {%- when AttributeSource::Subscription with (sub) %} {
                "$source": {
                    component: {% match sub.variable %}
                    {%- when Some with (var_name) -%}
                      template.getComponentName({{ var_name }}),
                    {%- when None -%}
                      {{ sub.component | json }},
                    {%- endmatch %}
                    path: {{ sub.path | json }},
                  {%- match sub.func %}
                  {%- when Some with (func_inner) %}
                    func: {{ func_inner | json }},
                  {%- when None %}
                  {%- endmatch %}
                }
            },
               {%- when AttributeSource::InputSource %}
               template.sourceOrValue({{ attr.dest_path | json }}, thisComponent),
            {%- endwhen %}
            {%- endmatch %}
            {%- endfor %}
        },
    };
    componentsToCreate.push({{ component.variable_name }});
    {% endfor %}
    // Assemble the return value; this shouldn't need to be modified.
    const create: Output["ops"]["create"] = {};
    for (const component of componentsToCreate) {
        const componentName = template.getComponentName(component);
        create[componentName] = component;
    }
    return {
        status: "ok",
        ops: {
            create,
        }
    }
}
