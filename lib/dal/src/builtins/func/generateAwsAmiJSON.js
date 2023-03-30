function generateAwsAmiJSON(input) {
    // Initialize the input JSON.
    const object = {
        "Filters": [],
    };

    if (input !== undefined && input !== null) {
        if (input.domain.ExecutableUsers) {
            object["ExecutableUsers"] = [input.domain.ExecutableUsers]
        }
    
        if (input.domain.Owners) {
            object["Owners"] = [input.domain.Owners]
        }
    
        if (input.domain.ImageId) {
            object["Filters"].push({
                "Name": "image-id",
                "Values": [input.domain.ImageId]
            });
        }
    
        if (input.domain.Filters) {
            for (const filter of input.domain.Filters) {
                object["Filters"].push({
                    "Name": filter.Name,
                    "Values": [filter.Value],
                });
            }
        }
    } 


    return {
        format: "json",
        code: JSON.stringify(object, null, '\t')
    };
}
