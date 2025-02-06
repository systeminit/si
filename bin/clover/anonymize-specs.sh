#!/bin/bash
set -e

POSITIONAL_ARGS=()
while [[ $# -gt 0 ]]; do
  case $1 in
    --remove-props)
      REMOVE_PROPS=,$2
      shift 2
      ;;
    -*|--*)
      echo "Unknown option $1"
      exit 1
      ;;
    *)
      break
      ;;
  esac
done

specs_dir=${1:-si-specs}
output_dir=$specs_dir/anonymized
echo "Anonymizing specs to $output_dir ..."
echo $REMOVE_PROPS
mkdir -p $output_dir
query=
for spec in $specs_dir/*.json; do
    spec_out=$output_dir/$(basename "$spec")
    jq 'del(.version,.createdAt,.schemas[].data.defaultSchemaVariant,(.schemas[].variants[] | .version,.data.version,.data.funcUniqueId),(..|select(type == "object")|(.uniqueId,.unique_id))'$REMOVE_PROPS')' -j < "$spec" > "$spec_out"
done
