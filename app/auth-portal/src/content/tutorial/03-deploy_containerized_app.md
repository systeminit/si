---
title: Deploy a containerized web application to an AWS EC2 Instance
---

## Deploy a containerized web application to an AWS EC2 instance
Have this tutorial open in one browser window and your System Initiative dev Workspace open in another. 

### The Workspace
Your System Initiative Workspace will look like this:

![Initial System Initiative Screen](/tutorial-img/03-deploy_containerized_app/initial_system_initiative_screen.png)

A few things worth noticing before we get started: 
* The primary navigation bar at the top of the screen has the Model icon selected
<img src="/tutorial-img/03-deploy_containerized_app/model_icon.png" alt="Model Icon" class="inline" width="5%" height="5%"/> indicating that you want to work on your infrastructure Model. We'll get to the other icon at the top of the screen, for Customization, in a later section of the tutorial. 
* In the center of the screen is a Workspace where you will design your Model, with a progress bar on top.
  * **What is a Model?**: In System Initiative, you build a Model of your infrastructure and applications. Think of this Model as a 1:1 simulation of the Resources you want to see in the real world, which provides immediate feedback on requirements and dependencies, and infers configuration on your behalf. You can safely iterate and experiment with your Model as much as you like without applying those changes to the Resources - it is purely a hypothesis about what you believe should exist.
* In the top left, we have automatically created and named a Change Set for you using the date and timestamp.
  * **What is a Change Set?**: All of the changes proposed by your Model are made within a Change Set, and when you decide that the Model reflects the reality you want and hit `Apply Changes`, System Initiative will take all the proposed changes in the Change Set and apply them in the correct order - creating, configuring, or destroying Resources to bring them in synch with your Model. You can think of a Change Set like a lightweight git branch that auto-rebases on main - where main is the version that you believe best reflects the reality you want. We call the 'main' version of your Model the `Head`.
* There are four panels on the sides of the Workspace. These can be resized up and down, left and right, to give you more space when you're working on them. 
  * `Diagram Outline Panel` (top left): lists of all the Assets in your Workspace, with a recap of their current status. 
  * `Asset Panel` (bottom left): a collection of the Assets you can use to build your Model.
  * `Changes Panel` (top right): The Proposed actions in your Change Set, and the changes already Applied.
  * `Selected Assets Panel` (bottom right): Once you select an Asset in the Workspace, you can use this panel to view its attributes, source code, and Resource data, and configure it according to your needs. 

### Modeling infrastructure

In this section, you will learn how to deploy a simple web application for [adopting cats](https://www.youtube.com/results?search_query=whiskers+r+we), which is in a [public docker container on Docker Hub](https://hub.docker.com/r/systeminit/whiskers), to an AWS EC2 Instance running [Fedora CoreOS](https://getfedora.org/en/coreos/download?tab=cloud_launchable&stream=stable&arch=x86_64).

> <Icon name="alert-triangle"></Icon>This tutorial will create resoures in AWS that have costs associated with them. 

#### Add and Configure a Docker Image

Scroll down the `Asset Panel` to find the Docker `Image` asset, click it, then click again to place it on the Workspace. You will see the progress bar update, indicating that System Initiative is updating the Model of your infrastructure.

When it finishes, click on the Docker `Image` you placed on the Workspace to select it. With the Docker `Image` asset selected, your Workspace will look something like this:

![Workspace with Docker Image](/tutorial-img/03-deploy_containerized_app/workspace_with_docker_image.png)

Let's take a minute to walk through all the information we're showing you.

![Docker image asset](/tutorial-img/03-deploy_containerized_app/docker_image_component.png)

* The Asset has a randomly generated name: in our case, si-8356.
* Below that is the 'type' of this Asset: a Docker Image.
* The green 'plus' icon indicates that this Asset was newly added to the Model in this Change Set.
* Below that are two Output Sockets - one named `Exposed Ports` and the other named `Container Image` (we will go into more detail about these later.)
* There is a red X warning in the bottom right corner of the Asset, which represents a Qualification failure.
  * **What is a Qualification?** 
 Qualifications are like built-in, real-time tests for your Model, letting you know when an Asset has not met all the requirements to function in the real world (i.e. whether it is 'qualified for use'). Qualifications can be built into specific Assets or into the Workspace itself (more on that in the Customization section of this tutorial). 

You can investigate the Qualification failure in the `Diagram Outline Panel`. Click on the red X to see that your Docker `Image` is not qualified because there is an error parsing the image name: your registry has no Docker `Image` with the name 'si-8356'.

![Qualification expanded](/tutorial-img/03-deploy_containerized_app/qualification_expanded.png)

To fix it, we need to configure the the Docker `Image` in the Model to point at a valid Docker Image. Go to the `Selected Assets Panel` on the right side of the screen, and drag it upwards to give yourself more room. When you do it will look something like this:

![Docker image details panel](/tutorial-img/03-deploy_containerized_app/docker_image_details_panel.png)

In the `Selected Assets Panel` (Attributes tab) change the `si/name`  of the Docker `Image` asset to `systeminit/whiskers`. Press 'Enter'. You'll see the progress bar update, and shortly after that the Qualification icon will turn green, both on the Workspace and in the `Diagram Outline Panel`.

Notice that, in addition to setting the `si/name` attribute, this action also sets `domain/image` to the same value. In System Initiative, Assets can infer configuration - either from their attributes or from relationships they have with other Assets. It's a powerful way to easily generate a correct configuration.

Our Docker `Image` exposes a web server running on port 80. Once again, go to the `Selected Assets Panel` (Attributes tab), scroll down to the `domain/ExposedPorts[0]` attribute, and click on the `+ Add to array` button. The progress bar will update, and you will see a new field for the ExposedPort.

Notice that, in addition to setting the `si/name` attribute, this action also sets `domain/image` to the same value. In System Initiative, Assets can infer configuration - either from their attributes or from relationships they have with other Assets. It's a powerful way to easily generate a correct configuration.

![Add to array](/tutorial-img/03-deploy_containerized_app/add_to_array.png)

Put the value `80/tcp` in this field, and hit 'Enter'.

![Added Exposed Port](/tutorial-img/03-deploy_containerized_app/whiskers_docker_attribute_panel.png)

The `80/tcp` syntax comes directly from Docker itself - like the image name, System Initiative maps the upstream behavior 1:1 - this allows you to easily transfer your existing knowledge about a given technology into System Initiative.

Notice that the Qualification for this Asset has turned from a red X to a green checkmark.

#### Add and Configure a CoreOS Butane asset: 

We want you to deploy your Docker Image on a [Fedora CoreOS](https://getfedora.org/en/coreos) system, in part because it has an excellent method for generating configuration for cloud instances called [Butane](https://coreos.github.io/butane/getting-started/).

You can use the Docker `Image` we just configured to automatically generate a Butane configuration for Fedora: Scroll down the `Asset Panel` to find the CoreOS `Butane` asset, then place it on the Workspace to the right of your Docker `Image`. Notice that your new `Butane` Asset has a `Container Image` input socket on the left side.

Connect the `Container Image` output socket of the Docker `Image` to the matching `Container Image` input socket on your new `Butane` asset by clicking on the output socket and dragging the line to the input socket.

![Connected sockets](/tutorial-img/03-deploy_containerized_app/connected_sockets.png)

When the progress bar has finished, you can see at a glance that the `Butane` asset is already qualified (the circular green checkmark), indicating that System Initiative believes this is a valid configuration.

Click the `Butane` asset so you can inspect its details in the `Selected Assets Panel`. Scroll down to the `domain/systemd/units/[0](unit)` attribute to investigate the unit files (you'll need to expand the size of the 'contents' area to read it). It should look something like this:
We want you to deploy your Docker Image on a [Fedora CoreOS](https://getfedora.org/en/coreos) system, in part because it has an excellent method for generating configuration for cloud instances called [Butane](https://coreos.github.io/butane/getting-started/).

You can use the Docker `Image` we just configured to automatically generate a Butane configuration for Fedora: Scroll down the `Asset Panel` to find the CoreOS `Butane` asset, then place it on the Workspace to the right of your Docker `Image`. Notice that your new `Butane` asset has a `Container Image` input socket on the left side.

Connect the `Container Image` output socket of the Docker `Image` to the matching `Container Image` input socket on your new `Butane` asset by clicking on the output socket and dragging the line to the input socket.

![Connected sockets](/tutorial-img/03-deploy_containerized_app/connected_sockets.png)

When the progress bar has finished, you can see at a glance that the `Butane` asset is already qualified (the circular green checkmark), indicating that System Initiative believes this is a valid configuration.

CClick the `Butane` asset so you can inspect its details in the `Selected Assets Panel`. Scroll down to the `domain/systemd/units/[0](unit)` attribute to investigate the unit files (you'll need to expand the size of the 'contents' area to read it). It should look something like this:

![Butane details](/tutorial-img/03-deploy_containerized_app/butane_details.png)

You'll see that System Initiative used the relationship between the Docker `Image` and the `Butane` instance to automatically infer that you wanted to create a [systemd unit file](https://www.freedesktop.org/software/systemd/man/systemd.unit.html), and wrote it for you - including translating the exposed ports to publish commands to [podman](https://podman.io/)!

`Butane` processes its configuration into a JSON format known as [Ignition]( https://coreos.github.io/ignition/examples/), and System Initiative automatically generates the Ignition data for you, based on the provided `Butane` configuration. To see it, look in the `Selected Assets Panel`, and click the Code tab.

![Code View](/tutorial-img/03-deploy_containerized_app/code_view.png)

#### Modeling the deployment of your CoreOS instance to AWS 

You'll need to pick a Region for your deployment. Select the AWS `Region` asset from the `Asset Panel`, and drop it on the Workspace to the right of your `Butane` configuration.

![Workspace with region](/tutorial-img/03-deploy_containerized_app/workspace_with_region.png)

Notice that this Asset looks different from the previous two! An AWS `Region` is a 'frame', while the previous two were 'components'. Frames are a way to organize or aggregate components in the Model for easier configuration or relationship creation. The `Region` frame is a _configuration_ frame, meaning that any Asset placed inside is automatically configured by it.

Resize the `Region` frame to be a little larger by clicking and dragging the corner of the frame. You can see in the lower right corner of the frame that it is not qualified - that's because you haven't decided which AWS Region to use.

If you investigate the Qualification failure in the `Diagram Outine Panel`, you'll learn that it appears because you haven't decided which AWS Region to use. Select the `Region` frame, and go to the `Selected Assets Panel` (Attributes tab). Set the `domain/region` attribute to `us-east-2`. When the progress bar finishes, you will see that your Qualification for this `Region` turns from a red X to a green check - you're good to go. System Initiative has also helpfully inferred the `si/name` of the `Region` for you!

![Set the region](/tutorial-img/03-deploy_containerized_app/set_the_region.png)

The application runs on an EC2 Instance, so let's model it. Select the AWS `EC2 Instance` asset from the `Asset Panel`, and click to place it inside the `Region` frame on the Workspace. You can then click and drag the `EC2 Instance` into the upper right corner of the `Region` frame. A couple of things to notice:

* You can see that the `Region` input socket on the `EC2 Instance` is already filled - configured by the `Region` frame in which it sits.
* If you investigate the Qualification failures in the `Diagram Outine Panel`, you'll see there are multiple issues here, and some will be resolved as we connect the EC2 instance input sockets to other Assets. Ignore these for now - we'll come back to them shortly.

Things should look like this:

![Ec2 Instance in Region Frame](/tutorial-img/03-deploy_containerized_app/ec2_instance_in_region_frame.png)

Let's work backward, connecting all the things your `EC2 Instance` needs in order to function. Your `Butane` configuration has a matching `User Data` output socket to your new `EC2 Instance`'s `User Data` input socket. Let's connect them now.

![Connected Inside a Region](/tutorial-img/03-deploy_containerized_app/connected_inside_a_region.png)

With the `EC2 Instance` selected, you can look to the `Selected Assets Panel` to check that the user data has been populated: 
* in the Attributes tab,  by looking at the `domain/UserData` field; or 
* in the Code tab, looking at the generated JSON code.

Working through the input sockets of your `EC2 Instance` from top to bottom, you can see that the next thing you will need is a Security Group ID. Grab a `Security Group` from the `Asset Panel`, drop it into the `Region` frame, and connect it to the `EC2 Instance`. I like to place the `Security Group` in the upper left corner of the `Region` frame.  The `Security Group` is already qualified with a green checkmark. Easy! 

A cat adoption website that nobody can reach from the outside world would be a sad, lonely website. To fix that, you need to add an Ingress rule to your `Security Group`. Select the AWS `Ingress` asset from the Asset Panel, and place it in the `Region` frame. I like to put it right in the middle of the Frame. 

Connect the `Security Group` to the `Ingress` rule. Then, observe that the `Ingress` asset has an input socket named `Exposed Ports` - and suspiciously, your Docker `Image` has an output socket with the same name. Connect the Docker `Image` to the `Ingress` rule. 

![Connect a Security Group and Ingress Rule](/tutorial-img/03-deploy_containerized_app/connect_a_security_group_and_ingress_rule.png)

Select the `Ingress` rule, and switch to the Code tab in the `Selected Assets Panel`. Check out how System Initiative has written the `IpPermissions` automatically. You now have a single configuration attribute, the ExposedPorts of your Docker `Image`, automatically configuring the Operating System, EC2 User Data, and your `Ingress` rule.

Select your Docker `Image`, go to the `Selected Assets Panel` (Attributes tab), and change the ExposedPort to <code>8080/tcp</code> rather than <code>80/tcp</code>.  You will see System Initiative calculate the scope of the update, then show that it is updating the configuration of all 3 impacted Assets on the Workspace!

That was fun, but let's switch it back to port 80. Select the Docker `Image`, go to the `Selected Assets Panel` (Attributes tab), and change ExposedPort back to 80/tcp.

Your `Ingress` rule has an orange Warning on Qualification status. Click on the warning in the `Diagram Outline Panel` and you can see why:

![Warning on Ingress](/tutorial-img/03-deploy_containerized_app/warning_on_ingress.png)

The configuration for this `Ingress` rule isn't technically valid (yet!) - for it to work, it needs to know the Security Group ID. You don't have one of those yet, because you are still in the modeling phase, and the AWS `Security Group` hasn't been created in the real world yet. Likewise for the other changes you've been modeling so far - they haven't created or configured real Resources in the world yet.

Once you tell System Initiative you want to make your changes real in the world by hitting the `Apply Changes` button, the `Security Group` will be created, and - because your `Ingress` rule is connected to your `Security Group` - the Security Group ID will pass through automatically and the warning will resolve itself.

Returning to your `EC2 Instance`'s input sockets, the next socket is the Key Name. AWS EC2 uses SSH Key Pairs to authenticate to the instances you boot. Add a `Key Pair` asset from the `Asset Panel` to your `Region` frame, right underneath yout `Ingress` rule, and connect it to your `EC2 Instance`. The `Key Pair` is already qualified with a green checkmark, so you know the configuration looks good. 

![Key Pair joins the party](/tutorial-img/03-deploy_containerized_app/key_pair_joins_the_party.png)

There is one more open input socket on the `EC2 Instance`: the `Image ID` socket. In AWS EC2, you use an AMIs ID as the Image ID.

Click the AWS `AMI` asset in the `Asset Panel`, then place it within the `Region` frame on the Workspace below the `Key Pair`. Connect the `Image ID` output socket on the `AMI` to the `Image ID` input socket on the `EC2 Instance`. Click on the Qualification failure in the `Diagram Outline Panel` to see that it needs an Image ID or a Filter. To address this, select the `AMI` and got to the `Selected Assets Panel` (Attributes tab) and populate <code>domain/ImageId</code> with <code>ami-0ed17ac79c5602c98</code> ([or the latest AMI for us-east-2 available on the CoreOS Download page](https://getfedora.org/en/coreos/download?tab=cloud_launchable&stream=stable&arch=x86_64)).  Hit Enter and see the Qualification for this `AMI` asset turn from a red X to a green checkmark.

![AMI joins the party](/tutorial-img/03-deploy_containerized_app/ami_joins_the_party.png)

Let's look again at the Qualifications on your `EC2 Instance` by clicking on the red X in the `Diagram Outline Panel`. 
* There is an orange Qualification warning, which will be resolved once the `Key Pair` is created. No action needed.  
* There is a Qualification failure, telling you that you need to configure the `/domain/InstanceType` attribute for your `EC2 Instance`. Let's fix that now. 

![Qualification warning and failure remain](/tutorial-img/03-deploy_containerized_app/Qualification_warning_and_failure_remain.png) 

Select the `EC2 Instance`, go to the `Selected Assets Panel`, (Attributes tab), scroll down to the the `domain/InstanceType` attribute, and set it to <code>t3.micro</code>. The red Qualification failure icon on this Asset will disappear, leaving just the orange Qualification warning in its place. 

Review the `Diagram Outline Panel`: 
* **Four Confirmation warnings (no action needed)**: Each of the four AWS Assets you used has a red tools Confirmation warning, which is telling you that the Model does not match the resources in reality. The model is proposing the create these Resources (you can see the 'create' actions in `Changes Panel`), and once you hit `Apply Changes` to create these resources these Confirmations will turn green.

* **Two Qualification warnings (no action needed)**: 
  * As mentioned earlier, there's an orange Qualification warning on the `Ingress` rule, which will be qualified for use once the `Security Group` is created and passes the Group ID through. 
  * There's also an orange Qualification warning on the `EC2 Instance` which will be qualified for use once the `Key Pair` is created. 

 ![Looking good](/tutorial-img/03-deploy_containerized_app/Looking_good.png)

### Apply your changes

In System Initiative, like in life, you can imagine as many potential realities as you would like - but in the end, you have to pick which one to manifest into the universe. This Model looks good - all the Qualifications are green (with one expected warning). Let's ship it.

To see the changes that your Model has proposed, expand the `Changes Panel` at the top right side of your Workspace and review the Proposed Changes tab. It'll look like this: 

![Proposed Changes](/tutorial-img/03-deploy_containerized_app/proposed_changes.png)

You'll notice that some of the Proposed Changes have a toggle. System Initiative has compared the Model you have configured to what is known about the Resources you have in the real world, and where a change is needed to bring the Model and reality into sync, these toggles give you the option to choose whether to go ahead and make that change.

You can get more information about the proposed actions by clicking on the chevrons in the Proposed Changes tab. Clicking through each of the four AWS Assets you used, you'll see that each has a 'recommendation' to 'create' the Resource in the real world.

In most cases, you will not worry about the toggles in the Proposed Changes tab because you want the Model and reality to be 1:1. If everything looks good, you'll click the green `Apply Changes` button in the upper-right corner to make the Model you have created real in the world.

A notification will pop up to remind you that some of the proposed changes will affect the external Resources or the model. Hit `Apply Changes` again to confirm the changes. 

![Apply Change Set](/tutorial-img/03-deploy_containerized_app/apply_change_set.png)

You will see a screen wipe, followed by a confetti cannon. The progress bar will start updating, and the Assets on the Workspace will have a spinner that indicates System Initiative is evaluating them against AWS. When it's done, the screen will look like this:

![Applied Changes](/tutorial-img/03-deploy_containerized_app/applied_changes.png)

Three things to notice about the screen in front of you:
* The name of the Change Set in the top left has changed to `head`. This means that the Model in your Workspace is read-only, and is a 1:1 representation of reality as it existed when the changes were last applied.

 * The `Changes Panel` is now showing only Applied Changes, and no longer contains a Proposed Changes tab.

* Looking at both the Workspace and the `Diagram Outline Panel`, the Confirmations that were red have now turned green - your Model now matches the Resources we expected in AWS.

Note that any new changes made to the Model will automatically open a new Change Set, and will appear in the `Changes Panel` as `Proposed Changes`.

You can inspect the data about your created Resources by clicking on the chevrons in the `Applied Changes` tab. For example, look at the Resource properties that were received as a result of creating your new `EC2 Instance`:

![Inspect Pending EC2 Instance in fix output](/tutorial-img/03-deploy_containerized_app/inspect_pending_ec2_instance_in_fix_output.png)

Of particular interest is the fact that your `EC2 Instance` has a 'State' of 'pending' - that means that it hasn't yet finished being created in AWS.

In order to see if you successfully started your application, you will need to hit the blue `Refresh Resources` <img src="/tutorial-img/03-deploy_containerized_app/refresh_button.png" alt="refresh button" class="inline"> button and, once AWS has provisioned your `EC2 Instance` the State will switch to running.

![Running state on Ec2 Instance](/tutorial-img/03-deploy_containerized_app/running_state_on_ec2_instance.png)

### Checking out your new website

To check and see if cat lovers can adopt some cats, you'll need the public IP address of your `EC2 Instance`.

Select your `EC2 Instance` and look in the `Selected Assets Panel` in the Resource tab, and find the `PublicIpAddress` in the JSON output.

![Finding the public IP Address](/tutorial-img/03-deploy_containerized_app/finding_the_public_ip_address.png)

Open a new tab in your web browser, and navigate to the address over http (no SSL for these kitties!). You should see the new website for Whiskers R We!

![Whiskers R We](/tutorial-img/03-deploy_containerized_app/whiskers_r_we.png)

Congratulations! Cats everywhere thank you.
 
___
 
# Summary version of the tutorial
If you want to walk through the action steps of this tutorial - without the context and explanation - you can follow the steps below.  

## 1. Add and Configure a Docker Image:
* **Add a Docker `Image`** asset to the Workspace from the `Asset Panel`. You will see the progress bar update, indicating that System Initiative is updating the Model of your infrastructure.
* **Investigate the Qualification failure** by clicking on the red X in the `Diagram Outline Panel`. The error arises because your registry has no Docker `Image` with that name (in the screenshot example it's 'si-2660'). 
* **Configure the Asset** 
  * Go to the `Selected Assets Panel` and change the `si/name`  of the Docker `Image` asset to `systeminit/whiskers`. Press 'Enter'. 
  * Go the `Selected Assets Panel`, scroll down to `domain/ExposedPorts[0]`, and click on the `+ Add to array` button. In the new field for the ExposedPort enter the value `80/tcp` and hit 'Enter'.
* The Qualification for this Asset turns from a red X to a green checkmark.

## 2. Add and Configure a CoreOS Butane asset: 
* **Add a CoreOS `Butane` asset** to the Workspace from `Asset Panel`.
* **Connect** the `Container Image` output socket of the Docker `Image` to the matching `Container Image` input socket on your new `Butane` asset. 
* **Check for Qualification failures**: The `Butane` asset is already qualified with a green checkmark. Easy! 

## 3. Add and Configure an AWS Region**: 
* **Add an AWS `Region` frame** to the Workspace from the `Asset Panel` and resize it to be a little larger by clicking and dragging the corner of the frame.
* **Investigate the Qualification failure** in the `Diagram Outine Panel`, and you'll learn that it appears because you haven't decided which AWS Region to use. 
* **Configure the Asset** to get rid of the Qualification failure. Select the `Region` frame, and go to the `Selected Assets Panel` (Attributes tab). Set the `domain/region` attribute to `us-east-2`. 
* The Qualification for this `Region` turns from a red X to a green checkmark. 

## 4. Add and Configure an EC2 Instance** 
* **Add an AWS `EC2 Instance`** and drop it inside the `Region` Frame. You can drag it up to the top right corner of the `Region` frame.
* **Connect** your `Butane` configuration's `User Data` output socket to your `EC2 Instance`'s `User Data` input socket. 
* **Investigate the Qualification failures** in the `Diagram Outine Panel`, and click 'view details' to see what needs to be done. There are multiple issues here, and some will be resolved as we connect the EC2 instance input sockets to other Assets. Ignore these for now - we'll come back to them shortly. 

## 5. Add and Configure a Security Group
* **Add a `Security Group`** to the top left of the Frame.
* **Connect** the `Security Group` to the `EC2 Instance`. 
* **Check for Qualification failures**: The `Security Group` is already qualified with a green checkmark. Easy! 

## 6. Add and Configure an Ingress rule**
* **Add an AWS `Ingress` asset** from the `Asset Panel`, and place in the middle of the `Region` frame.  
* **Connect** the `Security Group` to the `Ingress` rule, via their `Security Group ID` sockets. 
* **Connect** the Docker `Image` to the `Ingress` rule, via their `Exposed Ports` sockets. 
* **Investigate the orange Qualification warning** in the `Diagram Outline Panel`. It needs a Security Group ID, which will be passed through as soon as you `Apply Changes`  and the `Security Group` is created. Ignore for now. 

## 7. Add and Configure a Key Pair
* **Add a `Key Pair`** from the `Asset Panel` to your `Region` frame, right underneath yout `Ingress` rule
* **Connect** the `Key Pair` to your `EC2 Instance`.
* **Check for Qualification failures**: The `Key Pair` is already qualified with a green checkmark. Easy! 

## 8. Add and Configure an AMI asset  
* **Add an AWS `AMI` asset** to the `Region` frame beow the `Key Pair`.
* **Connnect** the `Image ID` output socket on the `AMI` to the `Image ID` input socket on the `EC2 Instance`.
* **Investigate the Qualification failure** in the `Diagram Outline Panel`. It needs an Image ID or a Filter. 
 * **Configure the `AMI` asset** to get rid of the Qualification failure. Going to the `Selected Assets Panel` (Attributes tab) and populate the Attribue <code>domain/ImageId</code> with <code>ami-0ed17ac79c5602c98</code> ([or the latest AMI for us-east-2 available on the CoreOS Download page](https://getfedora.org/en/coreos/download?tab=cloud_launchable&stream=stable&arch=x86_64)).  Hit Enter.
 * The Qualification for this `AMI` asset turns from a red X to a green checkmark.

## 9. Return to finish configuring the EC2 instance
* **Investigate the Qualification failure** in the `Diagram Outline Panel`. You need to configure the `/domain/InstanceType` attribute for your `EC2 Instance`. Let's fix that now. 
 * **Configure the `EC2 Instance`** to get rid of the Qualification failure. Select the `EC2 Instance`, go to the `Selected Assets Panel` (Attributes tab), scroll down to the the `domain/InstanceType` attribute, and set it to <code>t3.micro</code>. 
 * The Qualification for this `EC2 Instance` turns from a red X to a green checkmark.

## 10. Review the Diagram Outline Panel
* **Confirmation warnings (no action needed)**: Each of the four AWS Assets you used has a red tools Confirmation warning, which is telling you that the Model does not match the resources in reality. The model is proposing to create each of these Resources once you hit `Apply Changes` (you can see them listed in the `Changes Panel`). Once they are created, these Confirmations will turn green.
* **A single Qualification warning (no action needed)**: As mentioned earlier, there's an orange Qualification warning on the `Ingress` rule, which we expect to be qualified for use once the `Security Group` is created, because it will automatically pass the Group ID through.
* Looks like we're good to go! 

## 11. Apply your changes
* **Hit the `Apply Changes` button**, and hit it again to confirm the changes. 
* **Investigate the `Diagram Outline Panel`**: The red tools Confirmation warnings have now turned green - your Model now matches the Resources we expected in AWS.
* **Refresh resources**: Once the model has fully updated, hit the blue `Refresh Resources` <img src="/tutorial-img/03-deploy_containerized_app/refresh_button.png" alt="refresh button" class="inline"> button so that - once AWS has provisioned your `EC2 Instance` - the State will switch to running.

## 12. Check out your new website
* Select your `EC2 Instance` and look in the `Selected Assets Panel`, click the Code tab, and find the `PublicIpAddress` in the JSON output.
* Open a new tab in your web browser, and navigate to the address over http (no SSL for these kitties!). You should see the new website for Whiskers R We!
