---
title: Customizing System Initiative by writing TypeScript Code
---

## Customizing System Initiative by writing TypeScript Code

In the previous tutorial, you launched Whiskers R We running on [Fedora CoreOS](https://getfedora.org/en/coreos) EC2 instance. You learned how Qualifications are used to tell us if an Asset meets all the requirements to function in the real world - like a built-in, real-time test for your Model. 

The good folks at Whiskers R We want to ensure they are using the current stable version of the AMI for their AWS Region any time they launch a new instance in AWS. You will help them do that by writing a custom Qualification (using TypeScript) for all the AMIs in their Workspace.

### Create a new Change Set

Since you are changing how System Initiative works and updating your Model, you need to create a new Change Set. If you don't have any existing Change Sets, you will be prompted to create one automatically. If you have existing Change Sets, click the 'New Change Set' icon <img src="/tutorial-img/06-customizing/new_change_set.png" alt="New Change Set Icon" class="inline" width="5%" height="5%"/> in the navigation bar at the top of the screen.

You will see a dialog where you can give your Change Set a name, and click the `Create Change Set` button:

<img src="/tutorial-img/06-customizing/create_change_set_dialog.png" alt="Create Change Set Dialog" width="50%" height="50%"/> 

### Using Real-Time Multiplayer capabilities to your advantage

System Initiative is designed as a real-time multiplayer web application. What happens in your browser will immediately update all other users of the Workspace, enabling easy collaboration. You can take advantage of this capability when customizing System Initiative to see the impact of your customizations on your Model in real-time.

To get started, click the 'Copy Link' button in the main navigation at the top right of the screen, and use it to open System Inititative in a new browser window. 

![Copy Link](/tutorial-img/06-customizing/copy_link.png)

In one of the windows, click the Customize button <img src="/tutorial-img/06-customizing/customize.png" alt="customize button" class="inline" width="5%" height="5%"/> in the main navigation at the top of the screen. You should now have one browser window open to the `Model` screen and one open to the `Customize` screen:

![Customize screen](/tutorial-img/06-customizing/model_and_customize_screens.png)

### The Customize interface
Your Customize screen will look like this: 

![Customize screen](/tutorial-img/06-customizing/customize_screen.png)

A few things worth noticing before we get started:
* In the top left, you can see the same Change Set selector you saw in the Model screen. 
* Beneath that, there are three tabs - Assets, Functions, and Modules. This tutorial will focus on Functions. Select the `Function` tab now to open the `Function Panel`. 
  * At the top of the `Function Panel` is a `+ Function` button for creating new functions, followed by a search interface.
  * Below that there is a list of functions grouped by their type.
* The Workspace at center of the screen will show a tabbed code Editor (just as soon as you select a function to work on). 
* On the right side of the Workspace is the `Function Details Panel`, which will show the details of the currently selected function in the Editor.

### Looking at an existing function

Click the `search functions` box and type 'docker'. You will see the list of functions narrow to those that match the search criteria:

<img src="/tutorial-img/06-customizing/search_interface.png" alt="Search interface" width="70%" height="70%"/> 

Click the `Docker Image Exists` function from the Qualifications list, and the function will be loaded into the Editor, with its Properties listed in the `Function Details Panel`. 

![Docker image exists](/tutorial-img/06-customizing/docker_image_exists.png)

* At the top of the `Function Details Panel` there are two buttons, `Execute` and `Revert`. 
* Beneath that are the Attributes of the function: its Name, Display name, the Entrypoint (the name of the function to execute - you can have more than one function in a single 'file'), and a Description. 
* Underneath that, are two ways of associating this function with the Assets. We can apply it: 
  * directly on a single Asset (not selected in this case); or 
  * on __all__ the Assets of a given type. (you can see below that this 'Docker image exists' function is configured to run on all Docker Image Assets).

<img src="/tutorial-img/06-customizing/run_on_assets_of_type_docker.png" alt="Run_on_Docker assets" width="72%" height="72%"/> 

### Breakdown of a Qualification function

The Qualification function looks like this:

```js
async function qualificationDockerImageExists(component: Input): Promise<Output> {
  if (!component.domain?.image) {
    return {
      result: "failure",
      message: "no image available"
    }
  }
  const child = await siExec.waitUntilEnd("skopeo", ["inspect", "--override-os", "linux", "--override-arch", "amd64", `docker://${component.domain.image}`]);
  return {
    result: child.exitCode === 0 ? "success" : "failure",
    message: child.exitCode === 0 ? child.stdout : child.stderr,
  };
}
```

Functions in System Initiative are written in [TypeScript](https://www.typescriptlang.org/). The function signature tells you this is an [async TypeScript function](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/async_function), meaning it can use the async/await syntax in the function body.

The function checks to see if the Asset has an entry for the `domain/image` attribute, and if it does not, the function will return with a `{ result: "failure", message: "no image available" }` result. If the `domain/image` attribute is set, you can check the container's information using [skopeo]( https://github.com/containers/skopeo), and use its output and success or failure as your result.

_Note: Currently, functions are only partially sandboxed. In a future release of System Initiative, they will be fully isolated._

### Editing a function

Try updating this function with a slightly more helpful failure message, reminding the user that they haven't yet changed the `domain/image` from the pre-generated default hash, which is always in the format 'si-####'. 

The new function body should be:

```js
async function qualificationDockerImageExists(component: Input): Promise<Output> {
  if (!component.domain?.image || component.domain?.image.startsWith("si-")) {
    return {
      result: "failure",
      message: "no image available - set the domain/image attribute to something not auto-generated."
    }
  }
  const child = await siExec.waitUntilEnd("skopeo", ["inspect", "--override-os", "linux", "--override-arch", "amd64", `docker://${component.domain.image}`]);
  return {
    result: child.exitCode === 0 ? "success" : "failure",
    message: child.exitCode === 0 ? child.stdout : child.stderr,
  };
}
```

This function updates line 2 to read that it should return if the image is not set or if it starts with `si-`. It also updates line 5 to have the better, more useful, failure message.

Click the `Execute` button to run your function, and you should see the button spin and let you know that it finished without errors. If you have a syntax error, you will see it in the Editor and also see an error message immediately beneath the `Execute` button. Fix and press `Execute` again. 

![Execute error output](/tutorial-img/06-customizing/execute_error_output.png)

### Checking your work in the Model

In the Model Window, add a Docker `Image` Asset to the Workspace, select it, then click on the Qualification failure in the `Diagram Outline Panel`. 

![Qualification in Model Window](/tutorial-img/06-customizing/qualification_in_model_window.png)

Notice that the Qualification failure message has already been changed, and now helpfully tells us to set the `domain/image` attribute to something not autogenerated. Let's follow this advice. 

Select the Docker `Image` Asset, expand the `Selected Assets Panel`, set the `si/name` attribute of the Docker `Image` to "mysql", and press 'Enter'. You will see the progress bar update, and the Qualification will pass:

![Set the name](/tutorial-img/06-customizing/set_the_name.png)

### Making sure we use the latest stable Fedora CoreOS AMI

Now that you've edited an existing Qualification, let's use your newfound abilities to extend System Initiative in new ways. The application you deployed in the first tutorial ran on [Fedora CoreOS](https://getfedora.org/en/coreos). As a best practice, you want to be sure that you are always running the latest stable version of Fedora CoreOS available in a
given AWS Region. This is the kind of thing folks often try to do with "Policy as Code" frameworks. In System Initiative, you can write a custom Qualification to do it __in real time__.

In your Customization Window, remove your search filter from the search functions box, and close the tab for the 'Docker image exists' function by clicking the `X` on the tab:

<p align="left"><img src="/tutorial-img/06-customizing/close_the_tab.png" alt="Close the tab" width="72%" height="72%"/> 

Click the `+ Function` button, and select 'Qualification' from the dropdown menu. Your Customization Window should now look like this:

![Customization window ready to rock](/tutorial-img/06-customizing/customization_window_ready_to_rock.png)

In the `Function Details Panel` set the `Name` of your Qualification to "Only Use Latest Fedora CoreOS Stable AMIs". Enter the same for the `Display name`.  

Add a `Description` of "Ensures the AMI is using the latest stable Fedora CoreOS image in its region."

Set your new Qualification to run on all `AMI` assets in this Workspace: click on the `Select assets of type` dropdown, select `AMI`, and press the `+ Add` button. 

Configure the `Function Inputs` by checking the box for `Domain`, which points the function specifically at the Model in System Initiative. 

You should see:

<p align="left"><img src="/tutorial-img/06-customizing/function_details_complete.png" alt="Function details complete" width="72%" height="72%"/> 

Click the `Execute` button in the `Function Details Panel` to attach your new functionality to all the AWS AMI assets.

Before we add content to the Qualification, let's just check to make sure that it is working at a basic level. In your Model Window, click the AWS `AMI` asset in the `Asset Panel` and place it on the Workspace. Select the newly created `AMI`, and open the `Qualifications Panel` at the bottom of the screen (which replicates the information in the `Diagram Outline Panel` that we've been using) to see that your new Qualification is present:

![Model window with AMI](/tutorial-img/06-customizing/model_window_with_ami.png)

Back in the Customization Window, open the same `Qualifications Panel` at the bottom of the screen, and select your newly created AMI from the Components Menu in the bottom left of the Panel. That will allow you to see the impact of your new functionality on this Asset as you update it in real time.

![Customization window qualification](/tutorial-img/06-customizing/customization_window_qualification.png)

Your Customization Screen should now look like the following:

![Ready to customize](/tutorial-img/06-customizing/ready_to_customize.png)

Now try changing your Qualification function return value to report a failure rather than a success and see what happens. Put the following code into the Editor:

```js
async function qualification(component: Input): Promise<Output> {
  return {
    result: 'failure',
    message: 'Component qualified'
  };
}
```

Press the `Execute` button to see the Qualification status change to an failure state, in both the Customize and Model Windows.

![Qualification status failed on purpose](/tutorial-img/06-customizing/qualification_status_failed_on_purpose.png)

When authoring TypeScript functions, it's often convenient to use `console.log()` to print debug output. Add a console.log message to the first line of our function:

```js
async function qualification(component: Input): Promise<Output> {
  console.log("Hello from a custom qualification");
  return {
    result: 'failure',
    message: 'Component qualified'
  };
}
```

Press the `Execute` button, and when it has finished spinning, click the `View Details` link at the bottom of your custom Qualification. You will then see a modal dialog with the function's raw output in yellow:

![Console log example](/tutorial-img/06-customizing/console_log_example.png)

Close the modal window to return to the Editor.

### Fetching the list of stable Fedora CoreOS AMIs

Fedora CoreOS publishes [a list of stable artifacts for a variety of platforms and use cases via a simple JSON file](https://builds.coreos.fedoraproject.org/streams/stable.json). We recommend opening this in a new browser tab to help you navigate the data structure. You will primarily be concerned with the 'architectures.*.images.aws.regions' data.

The first step in writing your Qualification is to fetch that file and deserialize the response. Make your function look
like this:

```js
async function qualification(component: Input): Promise<Output> {
  const response = await fetch('https://builds.coreos.fedoraproject.org/streams/stable.json');
  const coreos = await response.json();
  const validArm64ImagesByRegion = coreos.architectures?.aarch64?.images?.aws?.regions;
  console.log("ARM 64 Regions", { validArm64ImagesByRegion });
  const validX8664ImagesByRegion = coreos.architectures?.x86_64?.images?.aws?.regions;
  console.log("X86 64 Regions", { validX8664ImagesByRegion });
  return {
    result: 'failure',
    message: 'Component qualified'
  };
}
```

* The code starts with a call to the [Fetch API](https://developer.mozilla.org/en-US/docs/Web/API/Fetch_API) - this call is asynchronous, and so you'll need to `await` it to get your response. 
* The next line then waits until the entire response body has been received, and then deserializes from JSON into a TypeScript Object. 
* The next four lines extract a map of all the current stable AMIs, indexed by region, and log the output. 

Press the `Execute` button, and then once again click the `View Details` link at the bottom of your custom Qualification, and you should see output similar to this:

![Ugly debug output](/tutorial-img/06-customizing/ugly_debug_output.png)

Ugly, but functional! You can dismiss the modal by pressing 'Escape' or clicking on the Workspace.

### Mapping the Qualification states

To write the rest of the code, you'll need to map out the various result states. Here they are in table form:

| Image Set | Region Set | Valid Image | Result  | Message                                                                  |
|-----------|------------|-------------|---------|--------------------------------------------------------------------------|
| N         | N          | N           | Warning | Cannot detect correct CoreOS Stable AMI without a region or ImageId set. |
| Y         | N          | N           | Failure | ImageId is set, but no region; Cannot validate ImageId. Set a region.    |
| Y         | Y          | N           | Failure | Incorrect CoreOS Stable AMI. Provide the correct AMIs                    |
| N         | Y          | N           | Failure | Incorrect CoreOS Stable AMI. Provide the correct AMIs                    |
| Y         | Y          | Y           | Success | Using current CoreOS Stable AMI                                          |

Update the code in the Editor with the following function, which translates the above into TypeScript:

```js
async function qualification(component: Input): Promise<Output> {
  const response = await fetch('https://builds.coreos.fedoraproject.org/streams/stable.json');
  const coreos = await response.json();
  const validArm64ImagesByRegion = coreos.architectures?.aarch64?.images?.aws?.regions;
  const validX8664ImagesByRegion = coreos.architectures?.x86_64?.images?.aws?.regions;
  let result: Output = {
    result: 'failure',
    message: 'Failed to qualify AMI; bad data.'
  };
  let validAmi = false;

  if (component.domain?.region) {
    const arm64Ami = validArm64ImagesByRegion[component.domain?.region]?.image;
    const x8664Ami = validX8664ImagesByRegion[component.domain?.region]?.image;

    if (component.domain?.ImageId) {
      if (validArm64ImagesByRegion && validX8664ImagesByRegion) {
        const validArm64Ami = arm64Ami == component.domain?.ImageId;
        const validx8664Ami = x8664Ami == component.domain?.ImageId;

        validAmi = validx8664Ami || validArm64Ami;

        if (validAmi) {
          result['result'] = "success";
          result['message'] = "Using current CoreOS Stable AMI";
        } else {
          result['result'] = "failure";
          result['message'] = `Incorrect CoreOS Stable AMI. Must be: x86_64, ${x8664Ami}; aarch64, ${arm64Ami}. You provided ${component.domain.ImageId}`;
        }
      }
    } else {
      result['result'] = "failure";
      result['message'] = `Incorrect CoreOS Stable AMI. Must be: x86_64, ${x8664Ami}; aarch64, ${arm64Ami}. No ImageId is set!`;
    }
  } else {
    if (component.domain?.ImageId) {
      result['result'] = "failure";
      result['message'] = "ImageId is set, but no region; Cannot validate ImageId. Set a region.";
    } else {
      result['result'] = "warning";
      result['message'] = "Cannot detect correct CoreOS Stable AMI without a region or ImageId set.";
    }
  }
  return result;
}
```

Press the `Execute` button, and let's start checking each result state. Your screen should look like this:

![Execute first state](/tutorial-img/06-customizing/execute_first_state.png)

You are currently in a state where you have neither a Region nor an ImageId set - therefore, your new Qualification warns you that it cannot qualify this Asset.

Switch to the Model Window, and set the `domain/ImageId` value to an invalid ImageId, like 'ami-000'.

![Set the ImageId](/tutorial-img/06-customizing/set_the_imageid.png)

Press the 'Enter' key, and you will see the Qualification update to tell you the ImageId couldn't be validated, and you still need to set a Region:

![Name state](/tutorial-img/06-customizing/name_state.png)

Now you can test the case where you have an invalid ImageID, but a valid Region. Set the `domain/region` attribute to 'us-east-2' and press 'Enter'.

The Qualification failure helpfully tells you that your ImageId is incorrect and tells you what the valid ImageIds are. 

![Invalid ImageId but Valid Region Qualification](/tutorial-img/06-customizing/invalid_imageid_but_valid_region_qualification.png)

To test the case where the ImageId is not set, but the Region is, you can hit the X button next to the ImageId, which updates your Qualification yet again:

![Valid region no image id](/tutorial-img/06-customizing/valid_region_no_image_id.png)

Our Qualification is still helpfully telling us which ImageIds are valid. Copy one of the ImageIds from the output of the Qualification, paste it into the `domain/ImageId` field, and press 'Enter'. Finally, all the Qualifications for this Asset are passing:

![Qualifications passing workspace](/tutorial-img/06-customizing/qualifications_passing_workspace.png)

Nice work! You've added a new Qualification to System Initiative that reflects the specific policy you needed.

### How does this thing work?

You ask excellent questions! Everything in System Initiative is a result of a TypeScript function execution. When you define a new Asset, you are defining the attributes it has, and setting functions for each value. As you have just seen, Qualifications and Confirmations are just functions. When System Initiative generates code for you - it's just a function that's reactive to the Asset's attributes. When Confirmations recommend actions, these, too, are functions that are reactive to both the attributes and the Resource for a given Asset.

System Initiative stitches this web of functions together into a reactive hyper-graph - allowing you to map any number of inputs to a function in the graph and then re-process the function if any of its inputs change. If you're familiar with how systems like [React](https://react.dev/) or [Vue](https://vuejs.org/) work, System Initiative is conceptually very similar.

We call it a hyper-graph because of Change Sets. Any individual Asset or function can be in multiple states at any time. When you create a new Change Set, think of it as creating a new place for a function to bind. If you haven't specified a binding for that particular Change Set, System Initiative falls back to whatever function is bound to `head`.

Everything in System Initiative is open and hackable - from the functions executed on the hyper-graph to the source code itself. Today you can create new Assets and other kinds of functions to build your own system in System Initiative, and we will add lots more functionality in the future, like integrated sharing and discovery functions (allowing you to build the Model up from a Resource, rather than define it upfront.) We hope you'll help us
explore what we can build together!

### Wewt!

You have successfully customized System Initiative. You learned:

* Everything in System Initiative is a TypeScript function, editable through the customize screen
* System Initiative is real-time and multiplayer
* You can use Qualifications to write "Policy as Code" that executes in real time
* Underneath System Initiative is a reactive hyper-graph of functions
* Everything in System Initiative is open and hackable - completely visible and actionable
