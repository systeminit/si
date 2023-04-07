---
title: Customizing System Initiative by writing JavaScript Code
---

## Customizing System Initiative by writing JavaScript Code

In the previous tutorial, you launched Whiskers R We running on [Fedora CoreOS](https://getfedora.org/en/coreos) EC2
instance. The generous folks at Whiskers R We want to ensure they are using the current stable version of the AMI for
their AWS Region any time they launch a new instance in AWS. You will help them do that by writing a custom
Qualification (using JavaScript) for all the AMIs in their Workspace.

First, ensure your development environment is running by following the instructions in
the '[Run a development instance of System Initiative](url)' section. Once you're ready, let's get started!

### Create a new Change Set

Since you are changing how System Initiative works and updating your model, you need to create a new Change Set. If you
don't have any existing Change Sets, you will be prompted to create one automatically. If you have existing Change Sets,
click the Change Set selector and select `- Create new Change Set -` from the drop-down.

![Change set drop down](tutorial-img/06-customizing/change_set_drop_down.png)

You will see the Create Change Set dialog. Give your Change Set a name, and click the Create Change Set button:

![Create change set dialog](tutorial-img/06-customizing/create_change_set_dialog.png)

### Using Real-Time Multiplayer capabilities to your advantage

System Initiative is designed as a real-time multiplayer web application. What happens in your browser will immediately
update all other users of the workspace, enabling easy collaboration. You can take advantage of this capability when
customizing System Initiative to see the impact of your customizations on your model in real-time.

To get started, click the 'Copy Link' button in the main navigation:

![Copy Link](tutorial-img/06-customizing/copy_link.png)

Open a new browser window, and paste the url into the url bar. The result should be two windows open to the same
workspace and Change Set:

![Two windows](tutorial-img/06-customizing/two_windows.png)

### The Customize interface

In one of the windows, click the Customize <img src="tutorial-img/06-customizing/customize.png" alt="Customize" width="63" />  button in
the main navigation. You should now have one browser window open to the `Model` as above and one open to `Customize`
that looks like this:

![Customize screen](tutorial-img/06-customizing/customize_screen.png)

We will refer to the window open to the `Model` screen as the 'Model Window', and the window open to the `Customize`
screen as the 'Customize Window' for the rest of this tutorial.

The Customize Windows left-side panel contains the same Change Set selector you saw in the Model screen. Beneath that,
are three tabs - Functions, Packages, and Assets. This tutorial will focus on the `Functions` tab. Below the tabs is a
button for creating new functions, followed by a search interface and a list of functions grouped by their type (the
functions are marked as 'builtin' because they ship with System Initiative). The center will display a tabbed code
editor (just as soon as you select a function to work on). The right side panel will show the details of the currently
selected function tab in the editor.

### Looking at an existing function

Click the `search functions` box and type 'docker'. You will see the list of functions narrow to those that match the
search criteria:

![Search interface](tutorial-img/06-customizing/search_interface.png)

Click 'Docker image exists' from the Qualifications list, and the function will be loaded into the editor:

![Docker image exists](tutorial-img/06-customizing/docker_image_exists.png)

You now have the 'Docker image exists' function loaded in the editor in the central panel and its properties listed in
the right side panel. At the top of the Properties tab in the right side panel, there are two buttons, Execute and
Revert (you may not have the Revert button yet; you will if you modify the source code.) Beneath that are the Attributes
of the function: its Name, the Entrypoint (the name of the function to execute - you can have more than one function in
a single 'file'), and a Description. Underneath that, are two ways of associating this function with the Assets we want
to run it on: directly on a given asset, or on __all__ the assets of a given type. This 'Docker image exists' function
is configured to run on all Docker Image assets.

### Breakdown of a Qualification function

The qualification function looks like this:

```js
async function qualificationDockerImageExists (component) {
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

Functions in System Initiative are written in [JavaScript](https://developer.mozilla.org/en-US/docs/Web/JavaScript). The
function signature tells you this is
an [async JavaScript function](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/async_function),
meaning it can use the async/await syntax in the function body.

Our function takes a single argument - a `component`, representing the specific asset that the function is being called
on. The component will have data that mirrors the attributes of the asset it is being called on.

Now check to see if the component has the `domain/image` attribute set on it. If it does not, the function will
immediately return with a `{ result: "failure", message: "no image available" }` result. If the `domain/image` attribute
is set, you can check the container's information using [skopeo]( https://github.com/containers/skopeo), and use its
output and success or failure as your result.

_Note: Currently, functions are only partially sandboxed. In a future release of System Initiative, they will be fully
isolated._

### Editing a function

Try updating this function with a slightly more helpful error message when the name is a pre-generated default (
si-XXXX). The new function body should be:

```js
async function qualificationDockerImageExists (component) {
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

You updated line 2 to read that it should return if the image is not set or if it starts with `si-`. You also updated
line 5 to have a better error message.

Click the Execute <img src="tutorial-img/06-customizing/execute_button.png" alt="Execute Button" width="124" /> button to run your function,
and make sure there are no errors. You should see the button spin and let you know that it finished without errors. If
you have a syntax error, you will see it in the editor and also see an error message immediately beneath the Execute
button:

![Execute error output](tutorial-img/06-customizing/execute_error_output.png)

You can then fix the error and click the Execute button to try again.

### Checking our work in the Model

In the Model Window, add a Docker Image asset to the canvas, select it, then open the Qualifications tab in the status
bar near the bottom of the screen.

![Model window for qualification](tutorial-img/06-customizing/model_window_for_qualification.png)

You will see in the Qualifications detail that our Docker Image Exists Qualification has already been changed, and our
new output is displayed:

![Edited qualification](tutorial-img/06-customizing/edited_qualification.png)

In the Customization Window, open the Qualifications tab in the status bar, and select the asset you just created. The
Customization window should look like this:

![Customization with qualifications](tutorial-img/06-customizing/customization_with_qualifications.png)

To ensure you get the correct output without an auto-generated name, set the `si/name` attribute of your Docker Image in
the Attributes tab of the details panel on the right to "mysql" in your Model Window. You will see the progress bar
update, and the Qualification will pass:

![Set the name](tutorial-img/06-customizing/set_the_name.png)

Notice that the Customization Window has also updated its Qualification status! Any changes made to the model will
automatically update for every user logged in to that workspace and viewing the same Change Set. This is the multiplayer
nature of System Initiative in action.

### Making sure we use the latest stable Fedora CoreOS AMI

Now that you've edited an existing Qualification, let's use your newfound abilities to extend System Initiative in new
ways. The application you deployed in the first tutorial ran on [Fedora CoreOS](https://getfedora.org/en/coreos). As a
best practice, you want to be sure that you are always running the latest stable version of Fedora CoreOS available in a
given AWS Region. This is the kind of thing folks often reach for "Policy as Code" frameworks to try and do. In System
Initiative, you can write a custom Qualification to do it __in real time__.

In your Customization Window, close the tab for the 'Docker image exists' function by clicking the `X` on the tab:

![Close the tab](tutorial-img/06-customizing/close_the_tab.png)

And remove your search filter from the search functions box:

![Remove the filter](tutorial-img/06-customizing/remove_the_filter.png)

Click the `+ Function` button, and select 'Qualification'.

![Add qualification drop down](tutorial-img/06-customizing/add_qualification_drop_down.png)

Your Customization Window should now look like this:

![Customization window ready to rock](tutorial-img/06-customizing/customization_window_ready_to_rock.png)

In the Properties tab in the right side details panel, set the Name of your Qualification to "Only Use Latest Fedora
CoreOS Stable AMIs".

![Properties tab](tutorial-img/06-customizing/properties_tab.png)

Add a Description of "Ensures the AMI is using the latest stable Fedora CoreOS image in its region."

![Description filled in](tutorial-img/06-customizing/description_filled_in.png)

Your new Qualification should run on all assets of type AMI in this Workspace. To enable that, click the 'select assets
of type' dropdown beneath the 'Run on Assets of Type' header. It may be helpful to scroll the panel all the way to the
bottom before clicking the dropdown.

![Run on assets of type](tutorial-img/06-customizing/run_on_assets_of_type.png)

Select 'AMI' and press the `+ Add` button. You should see:

![Click add](tutorial-img/06-customizing/click_add.png)

Click the Execute button in the Properties tab to attach your new functionality to all the AWS AMI assets.

In your Model Window, click the AWS AMI asset in the Asset Palette, and click again to place it on the canvas. Select
the newly created AMI, and you will see in the Qualifications detail that your new Qualification is present:

![Model window with AMI](tutorial-img/06-customizing/model_window_with_ami.png)

Back in the Customization Window, select your newly created AMI from the Components Menu in the Qualifications tab in
the status bar - that will allow you to see the impact of your new functionality in real-time.

![Customization window qualification](tutorial-img/06-customizing/customization_window_qualification.png)

Your Customization Screen should now look like the following:

![Ready to customize](tutorial-img/06-customizing/ready_to_customize.png)

Now try changing your qualification functions return value to report a failure rather than a success and see what
happens. Put the following code into the editor:

```js
async function qualification (component) {
  return {
    result: 'failure',
    message: 'Component qualified'
  };
}
```

Press the Execute button to see the Qualification status change, in both the Customize and Model Windows.

![Qualification status failed on purpose](tutorial-img/06-customizing/qualification_status_failed_on_purpose.png)

When authoring JavaScript functions, it's often convenient to use `console.log()` to print debug output. Add a
console.log message to the first line of our function:

```js
async function qualification (component) {
  console.log("Hello from a custom qualification");
  return {
    result: 'failure',
    message: 'Component qualified'
  };
}
```

Press the Execute button, and when it has finished spinning, click the 'View Details' link at the bottom of our custom
qualification. You will then see a modal dialog with the function's raw output in yellow:

![Console log example](tutorial-img/06-customizing/console_log_example.png)

Close the modal window to return to the Function Editor.

### Fetching the list of stable Fedora CoreOS AMIs

Fedora CoreOS
publishes [a list of stable artifacts for a variety of platforms and use cases via a simple JSON file](https://builds.coreos.fedoraproject.org/streams/stable.json).
We recommend opening this in a new browser tab to help you navigate the data structure. You will primarily be concerned
with the 'architectures.*.images.aws.regions' data.

The first step in writing our Qualification is to fetch that file and deserialize the response. Make your function look
like this:

```js
async function qualification (component) {
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

The code starts with a call to the [Fetch API](https://developer.mozilla.org/en-US/docs/Web/API/Fetch_API) - this call
is asynchronous, and so you'll need to `await` it to get your response. The next line then waits until the entire
response body has been received, and then deserializes from JSON into a JavaScript Object.

The next four lines extract a map of all the current stable AMIs, indexed by region, and log the output. Press the
Execute button, then View Details, and you should see output similar to this:

![Ugly debug output](tutorial-img/06-customizing/ugly_debug_output.png)

Ugly, but functional! You can dismiss the modal by pressing 'Escape'.

### Mapping the Qualification states

To write the rest of the code, you'll need to map out the various result states. Here they are in table form:

| Image Set | Region Set | Valid Image | Result  | Message                                                                  |
|-----------|------------|-------------|---------|--------------------------------------------------------------------------|
| N         | N          | N           | Warning | Cannot detect correct CoreOS Stable AMI without a region or ImageId set. |
| Y         | N          | N           | Failure | ImageId is set, but no region; Cannot validate ImageId. Set a region.    |
| Y         | Y          | N           | Failure | Incorrect CoreOS Stable AMI. Provide the correct AMIs                    |
| N         | Y          | N           | Failure | Incorrect CoreOS Stable AMI. Provide the correct AMIs                    |
| Y         | Y          | Y           | Success | Using current CoreOS Stable AMI                                          |

Update the code in the editor with the following function, which translates the above into JavaScript:

```js
async function qualification (component) {
  const response = await fetch('https://builds.coreos.fedoraproject.org/streams/stable.json');
  const coreos = await response.json();
  const validArm64ImagesByRegion = coreos.architectures?.aarch64?.images?.aws?.regions;
  const validX8664ImagesByRegion = coreos.architectures?.x86_64?.images?.aws?.regions;
  let result = {
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

Press the Execute button, and let's start checking each result state. Your screen should look like this:

![Execute first state](tutorial-img/06-customizing/execute_first_state.png)

You are currently in a state where you have neither a region nor an ImageId set - therefore, your new qualification
warns you that it cannot qualify this asset.

Switch to the Model Window, and set the 'domain/ImageId' value to 'ami-000'.

![Set the ImageId](tutorial-img/06-customizing/set_the_imageid.png)

Press the 'enter' key, and you will see the qualification update:

![Name state](tutorial-img/06-customizing/name_state.png)

Now you can test the case where you have an invalid ImageID, but a valid Region. Set the `domain/region` attribute to
'us-east-2' and press 'enter'.

![Invalid ImageId but Valid Region](tutorial-img/06-customizing/invalid_imageid_but_valid_region.png)

The resulting qualification:

![Invalid ImageId but Valid Region Qualification](tutorial-img/06-customizing/invalid_imageid_but_valid_region_qualification.png)

To test the case where the ImageId is not set, but the region is, you can hit the X button next to the ImageId:

![Unset image id](tutorial-img/06-customizing/unset_image_id.png)

Which updates your Qualification yet again:

![Valid region no image id](tutorial-img/06-customizing/valid_region_no_image_id.png)

Copy and paste one of the provided ImageIds from the qualifications output into the `domain/ImageId` field:

![Copy and pase the image id](tutorial-img/06-customizing/copy_and_pase_the_image_id.png)

And see that in our final case, all the qualifications for this image are passing:

![Qualifications passing workspace](tutorial-img/06-customizing/qualifications_passing_workspace.png)

Nice work! You've added a new Qualification to System Initiative that reflects the specific policy you needed.

### How the hell does this thing work?

You ask excellent questions! Everything in System Initiative is a result of a JavaScript function execution. When you
define a new asset, you are defining the attributes it has, and setting functions for each value. As you have just seen,
things like validations and qualifications are just functions. When System Initiative generates code for you - it's just
a function that's reactive to the asset's attributes. When Confirmations recommend actions, these, too, are functions
that are reactive to both the attributes and the resource for a given asset.

System Initiative stitches this web of functions together into a reactive hyper-graph - allowing you to map any number
of inputs to a function in the graph and then re-process the function if any of its inputs change. If you're familiar
with how systems like [React](https://react.dev/) or [Vue](https://vuejs.org/) work, it's conceptually very similar.

We call it a hyper-graph because of Change Sets. Any individual asset or function can be in multiple states at any time.
When you create a new Change Set, think of it as creating a new place for a function to bind. If you haven't specified a
binding for that particular Change Set, we fall back to whatever function is bound to HEAD.

Everything in System Initiative is open and hackable - from the functions executed on the hyper-graph to the source code
itself. We will add lots of functionality in the future, like integrated sharing, new asset creation, and discovery
functions (allowing you to build the model up from a resource, rather than define it upfront.) We hope you'll help us
explore what we can build together!

### Wewt!

You have successfully customized System Initiative. You learned:

* Everything in System Initiative is a JavaScript function, editable through the customize screen
* System Initiative is real-time and multiplayer
* You can use Qualifications to write "Policy as Code" that executes in real time
* Underneath System Initiative is a reactive hyper-graph of functions
* Everything in System Initiative is open and hackable - completely visible and actionable