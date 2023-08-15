---
title: Deploy a containerized web application to an AWS EC2 Instance
---

## Deploy a containerized web application to an AWS EC2 instance
Have this tutorial open in one browser window and your System Initiative workspace open in another.

If you already know how System Initiative works and want to go straight to the tutorial summary, [click here](#summary-version-of-the-tutorial).

### The workspace
Your System Initiative workspace will look like this:

![Initial System Initiative Screen](/tutorial-img/03-deploy_containerized_app/initial_system_initiative_screen.png)

A few things worth noticing before we get started:
* The primary navigation bar at the top of the screen has the Model icon selected
<img src="/tutorial-img/03-deploy_containerized_app/model_icon.png" alt="Model Icon" class="inline" width="5%" height="5%"/> indicating that you want to work on your infrastructure Model. We'll get to the other icon at the top of the screen, for Customization, in a later section of the tutorial.
* In the center of the screen is a Canvas where you will design your Model, with a progress bar on top.
  * **What is a Model?**: In System Initiative, you build a Model of your infrastructure and applications to simulate the Resources you want to see in the real world. This Model provides immediate feedback on requirements and dependencies, and infers configuration on your behalf. You can safely iterate and experiment with your Model as much as you like without applying those changes to the Resources - it is purely a hypothesis about what you believe should exist.
* In the top left, you can see that you're on `head`, which is the 'main' version of your model. As soon as you make a change to your Model on the Canvas, System Initiative will automatically create a Change Set for you, giving it a name derived from the date and timestamp.
  * **What is a Change Set?**: All of the changes proposed by your Model are made within a Change Set, and when you decide that the Model reflects the reality you want and click `Apply Changes`, System Initiative will take all the proposed changes in the Change Set and apply them in the correct order - creating, configuring, or destroying Resources to bring them in sync with your Model.
  * If you're familiar with git, you can think of a Change Set like a lightweight git branch that auto-rebases on main - where main is the version you believe best reflects the reality you want. We call the 'main' version of your Model the `head`.
* There are four panels on the sides of the Canvas. These can be resized up and down, left and right, to give you more space when you're using them.
  * `Diagram Outline Panel` (top left): lists of all the Assets currently being used in your model, along with their current status.
  * `Asset Panel` (bottom left): a collection of the Assets you can use to build your Model.
  * `Changes Panel` (top right): The proposed actions in your Change Set, or a history of the changes already applied.
  * `Selected Assets Panel` (bottom right): Once you select an Asset on the Canvas, you can use this panel to view its attributes, source code, and Resource data; and configure it according to your needs.

### Modeling infrastructure

In this section, you will learn how to deploy a simple web application for [adopting cats](https://www.youtube.com/results?search_query=whiskers+r+we), which is in a [public docker container on Docker Hub](https://hub.docker.com/r/systeminit/whiskers), to an AWS EC2 Instance running [Fedora CoreOS](https://getfedora.org/en/coreos/download?tab=cloud_launchable&stream=stable&arch=x86_64).

> <Icon name="alert-triangle"></Icon>This tutorial will create resources in AWS that have costs associated with them.

#### Add and Configure a Docker Image

Scroll down the `Asset Panel` to find the `Docker Image` asset, click it, then click again to place it on the Canvas. You will see the progress bar update, indicating that System Initiative is updating the Model of your infrastructure.

When it finishes, click on the `Docker Image` you placed on the Canvas to select it. With the Docker `Image` asset selected, your Canvas will look something like this:

![Canvas with Docker Image](/tutorial-img/03-deploy_containerized_app/workspace_with_docker_image.png)

Let's take a minute to walk through all the information we're showing you.

![Docker image asset](/tutorial-img/03-deploy_containerized_app/docker_image_component.png)

* The Asset has a randomly generated name: in our case, si-8356.
* Below that is the 'type' of this Asset: a Docker Image.
* The green 'plus' icon indicates that this Asset was newly added to the Model in this Change Set.
* Below that are two Output Sockets - one named `Exposed Ports` and the other named `Container Image` (we will go into more detail about these later.)
* There is a red X icon in the bottom right corner of the Asset, which represents a Qualification failure.
  * **What is a Qualification?**
 Qualifications are like built-in, real-time tests for your Model, letting you know whether an Asset has met all the requirements to function in the real world (i.e., whether it is 'qualified for use'). Qualifications can apply to specific Assets or all Assets of a given type (more on that in the Customization section of this tutorial).

You can investigate the Qualification failure in the `Diagram Outline Panel`. Click on the red X to see that your `Docker Image` is not qualified because there is an error parsing the image name: the default registry (docker hub) has no Docker `Image` with the name 'si-8356'.

![Qualification expanded](/tutorial-img/03-deploy_containerized_app/qualification_expanded.png)

To fix it, we need to configure the `Docker Image` asset to point at a valid Docker Image. Go to the `Selected Assets Panel` on the right side of the screen, and drag it upwards to give yourself more room. When you do, it will look something like this:

![Docker image details panel](/tutorial-img/03-deploy_containerized_app/docker_image_details_panel.png)

In the `Selected Assets Panel` change the `si/name` of the `Docker Image` asset to `systeminit/whiskers`. Press 'Enter'. You'll see the progress bar update, and shortly after that the Qualification icon will turn green, both on the canvas and in the `Diagram Outline Panel`.

Notice that, in addition to setting the `si/name` attribute, this action also sets `domain/image` to the same value. In System Initiative, assets can infer configuration - either from their attributes or from relationships they have with other assets. It's a powerful way to generate a correct configuration easily.

Our `Docker Image` exposes a web server running on port 80. Once again, go to the `Selected Assets Panel`, scroll down to the `domain/ExposedPorts[0]` attribute, and click the `+ Add to array` button. The progress bar will update, and you will see a new field for the ExposedPort.

![Add to array](/tutorial-img/03-deploy_containerized_app/add_to_array.png)

Put the value `80/tcp` in this field, and press 'Enter'.

![Added Exposed Port](/tutorial-img/03-deploy_containerized_app/whiskers_docker_attribute_panel.png)

The `80/tcp` syntax comes directly from Docker itself - like the image name, System Initiative maps the upstream behavior 1:1 - this allows you to easily transfer your existing knowledge about a given technology into System Initiative.

Notice that the Qualification for this Asset has turned from a red X to a green checkmark - our `Docker Image` is looking good!

#### Add and Configure a CoreOS Butane asset:

We want you to deploy your Docker Image on a [Fedora CoreOS](https://getfedora.org/en/coreos) system, in part because it has an excellent method for generating configuration for cloud instances called [Butane](https://coreos.github.io/butane/getting-started/).

You can use the `Docker Image` we just configured to automatically generate a Butane configuration for Fedora: Scroll down the `Asset Panel` to find the CoreOS `Butane` asset, then place it on the Canvas to the right of your Docker `Image`. Notice that your new `Butane` Asset has a `Container Image` input socket on the left side.

Connect the `Container Image` output socket of the `Docker Image` to the matching input socket on your new `Butane` asset by clicking and dragging the line between them.

![Connected sockets](/tutorial-img/03-deploy_containerized_app/connected_sockets.png)

When the progress bar has finished, you can see at a glance that the `Butane` asset is already qualified (the circular green checkmark), indicating that System Initiative believes this is a valid configuration.

Click the `Butane` asset to inspect its details in the `Selected Assets Panel`. Scroll down to the `domain/systemd/units/[0](unit)` attribute to investigate the unit files (you'll need to expand the size of the 'contents' area to read it). It should look like this:

![Butane details](/tutorial-img/03-deploy_containerized_app/butane_details.png)

You'll see that System Initiative used the relationship between the `Docker Image` and the `Butane` instance to automatically infer that you wanted to create a [systemd unit file](https://www.freedesktop.org/software/systemd/man/systemd.unit.html), and wrote it for you - including translating the exposed ports to publish arguments to [podman run](https://podman.io/)!

`Butane` processes its configuration into a JSON format known as [Ignition]( https://coreos.github.io/ignition/examples/), and System Initiative automatically generates the Ignition data for you based on the provided `Butane` configuration. To see it, look in the `Selected Assets Panel`, and click the Code tab.

![Code View](/tutorial-img/03-deploy_containerized_app/code_view.png)

System Initiative inferred the correct configuration for our CoreOS instance, and generated the neccessary code automatically.

#### Modeling the deployment of your CoreOS instance to AWS

In order to get things running in AWS, you'll need to pick a `Region` for your deployment. Select the `AWS Region` asset from the `Asset Panel`, and drop it on the Canvas to the right of your `Butane` configuration.

![Canvas with region](/tutorial-img/03-deploy_containerized_app/canvas_with_region.png)

Notice that this Asset looks different from the previous two. An `AWS Region` is a configuration 'frame,' meaning that any asset placed inside is automatically configured by it.

You will see a red Qualification failure on the `Region`. Investigate in the `Diagram Outline Panel` and you'll see the cause: you haven't yet decided which AWS Region to use. With the `Region` frame selected, go to the `Selected Assets Panel`, and set the `domain/region` attribute to `us-east-2`. When the Qualification for this `Region` turns from a red X to a green check - you're good to go. Notice System Initiative has also helpfully inferred the `si/name` of the `Region` for you!

Resize the `Region` frame to be larger by clicking and dragging the corner of the frame.

![Set the region](/tutorial-img/03-deploy_containerized_app/set_the_region.png)

The application runs on an `EC2 Instance`, so let's model it. Select the `AWS EC2 Instance` asset from the `Asset Panel`, and click to place it inside the `Region` frame on the Canvas. You can then click and drag the `EC2 Instance` into the upper right corner of the `Region` frame.

* Investigate the two red icons on your `EC2 Instance` by clicking on the `Diagram Outline Panel`:
  * A red X Qualification failure: like the one we saw earlier, which means you need to configure the Asset. You can ignore this for now - we'll come back to it once we have connected the `EC2 Instance` input sockets to other Assets.
  * A red tools 'Changes' notification: which just notifies you that this Resource does not exist yet. It will exist once you click `Apply Changes` (a little later, once we're finished modeling), and this notification will resolve.
* You can see that the `Region` input socket on the `EC2 Instance` is already filled - it was configured by the `Region` frame in which it sits.

Things should look like this:

![Ec2 Instance in Region Frame](/tutorial-img/03-deploy_containerized_app/ec2_instance_in_region_frame.png)

Let's work backward, connecting all the things your `EC2 Instance` needs in order to function. Your `Butane` configuration has a matching `User Data` output socket to your new `EC2 Instance`'s `User Data` input socket. Go ahead and connect them now.

![Connected Inside a Region](/tutorial-img/03-deploy_containerized_app/connected_inside_a_region.png)

With the `EC2 Instance` selected, you can look to the `Selected Assets Panel` to check that the user data has been populated: in the Attributes tab, look at the `domain/UserData` field; and in the Code tab, look at the generated JSON code.

Let's work through the input sockets of your `EC2 Instance` from top to bottom:
The next socket is for a __Security Group__ ID.
* Grab a `Security Group` from the `Asset Panel`, drop it into the `Region` frame (I like to place the `Security Group` in the upper left corner of the `Region` frame).
* Connect it to the `EC2 Instance`.
* The `Security Group` is already qualified with a green checkmark. Easy!
* The `Security Group` has a red tools 'Changes' notification - like the one we saw earlier on the `EC2 Instance` - which lets you that this Resource does not exist yet. This will resolve later, when you click `Apply Changes`.

![Security Group Added](/tutorial-img/03-deploy_containerized_app/security_group_added.png)

A cat adoption website that nobody can reach from the outside world would be a sad, lonely website. To fix that, you need to add an __Ingress rule__ to your `Security Group`.
* Select the `AWS Ingress` asset from the Asset Panel, and place it in the `Region` frame (I like to put it right in the middle of the Frame).
* Connect the `Security Group` to the `Ingress` rule.
* Notice that the `Ingress` asset has an input socket named `Exposed Ports` - which matches an output socket on your `Docker Image`. Connect the Docker `Image` to the `Ingress` rule.

![Connect a Security Group and Ingress Rule](/tutorial-img/03-deploy_containerized_app/connect_a_security_group_and_ingress_rule.png)

Investigate the `Ingress` rule in the `Diagram Outline Panel`:
* Your `Ingress` rule has a red tools 'Changes' notification. You probably know by now that this is just letting you know the Resource doesn't exist yet.
* It also has has an orange Qualification warning. Click on the warning icon in the `Diagram Outline Panel` to see why:

![Warning on Ingress](/tutorial-img/03-deploy_containerized_app/warning_on_ingress.png)

The configuration for this `Ingress` rule isn't qualified for use (yet!) - for it to work, it needs to know the Security Group ID. You don't have one of those yet, because the AWS `Security Group` doesn't exist yet (it won't exist until you've finished modeling and clicked `Apply Changes`). Since your `Ingress` rule is connected to your `Security Group` in the model, once the `Security Group` exists the Security Group ID will pass through automatically, and the warning will resolve itself.

Let's investigate some of the automation in your model:

* Select the `Ingress` rule, and switch to the Code tab in the `Selected Assets Panel`. Check out how System Initiative has written the `IpPermissions` automatically. You now have a single configuration attribute, the ExposedPorts of your `Docker Image`, automatically configuring the Operating System, EC2 User Data, and your `Ingress` rule.
* Select your `Docker Image`, go to the `Selected Assets Panel` (Attributes tab), and change the ExposedPort to <code>8080/tcp</code> rather than <code>80/tcp</code>. You will see System Initiative calculate the scope of the update, then show that it is updating the configuration of all 4 impacted Assets on the Canvas! That was fun, but let's switch it back to port 80 - those cat's won't adopt themselves! Select the Docker `Image`, go to the `Selected Assets Panel`, and change ExposedPort back to <code>80/tcp</code>.

Returning to your `EC2 Instance`'s input sockets, the next socket is the Key Name. AWS EC2 uses SSH __Key Pairs__ to authenticate to the instances you boot.
* Add a `Key Pair` asset from the `Asset Panel` to your `Region` frame, right underneath your `Ingress` rule.
* Connect it to your `EC2 Instance`.
* Your `Key Pair` has a red tools 'Changes' notification to remind you that the Resource doesn't exist yet.
* The `Key Pair` is already qualified with a green checkmark, so you know the configuration looks good.

![Key Pair joins the party](/tutorial-img/03-deploy_containerized_app/key_pair_joins_the_party.png)

There is one more open input socket on the `EC2 Instance`: the Image ID socket. In AWS EC2, you use an __AMI ID__ as the Image ID.

* Click the AWS `AMI` asset in the `Asset Panel`, then place it within the `Region` frame on the Canvas below the `Key Pair`.
* Connect the `Image ID` output socket on the `AMI` to the corrresponding input socket on the `EC2 Instance`.
* Click on the Qualification failure in the `Diagram Outline Panel` to see that it needs an Image ID. To address this, select the `AMI` and go to the `Selected Assets Panel` and populate <code>domain/ImageId</code> with <code>ami-0ed17ac79c5602c98</code> ([or the latest AMI for us-east-2 available on the CoreOS Download page](https://getfedora.org/en/coreos/download?tab=cloud_launchable&stream=stable&arch=x86_64)). Click Enter and see the Qualification for this `AMI` asset turn from a red X to a green checkmark.

![AMI joins the party](/tutorial-img/03-deploy_containerized_app/ami_joins_the_party.png)

Let's look again at the Qualifications on your `EC2 Instance` by clicking the red X in the `Diagram Outline Panel`.
* There is an orange Qualification warning, which will be resolved once the `Key Pair` is created - no action is needed.
* There is a Qualification failure, telling you that you need to configure the `/domain/InstanceType` attribute for your `EC2 Instance`. Let's fix that now.

![Qualification warning and failure remain](/tutorial-img/03-deploy_containerized_app/Qualification_warning_and_failure_remain.png)

Select the `EC2 Instance`, go to the `Selected Assets Panel`, scroll down to the `domain/InstanceType` attribute, and set it to <code>t3.micro</code>. The red Qualification failure icon on this Asset will disappear, leaving just the orange Qualification warning in its place.

Review the `Diagram Outline Panel`:
* **Four 'Changes' notifications (red tools)**: (no action needed)
  * Each of the four AWS Assets you used has a red tools 'Changes' notification, which are telling you that these resource does not exist yet.
  * These Resources will be created once you click `Apply Changes` in a moment. Once they exist, these notifications will turn green.

* **Two Qualification warnings (orange exclamation points)**: (no action needed)
  * As mentioned earlier, there's an orange Qualification warning on the `Ingress` rule, which will be qualified for use once the `Security Group` is created and passes the Group ID through.
  * There's also an orange Qualification warning on the `EC2 Instance`, which will be qualified for use once the `Key Pair` is created.


 ![Looking good](/tutorial-img/03-deploy_containerized_app/looking_good.png)

### Apply your changes

In System Initiative, like in life, you can imagine as many potential realities as you would like - but in the end, you have to pick which one to manifest into the universe. This Model looks good - all the Qualifications are green (with two expected warnings). Let's ship it.

To see the changes you have proposed in this change set, expand the `Changes Panel` at the top right side of your Canvas and review the Proposed Changes tab. It'll look like this:

![Proposed Changes](/tutorial-img/03-deploy_containerized_app/proposed_changes.png)

System Initiative keeps track of every change you made, and the actions it will take when you apply them to reality. Each of these actions has a green toggle next to them, allowing you to decide if you want to take them now, or leave them unresolved (perhaps to be done at a later time.)

Everything looks good - click the green `Apply Changes` button in the upper-right corner to apply your changes to the real world. System Initiative will prompt you once again to confirm you want to take the actions in your change set - click `Apply Changes` again to confirm.

![Apply Change Set](/tutorial-img/03-deploy_containerized_app/apply_change_set.png)

The progress bar will start updating, and the Assets on the Canvas will have a spinner that indicates System Initiative is evaluating them against AWS; creating and configuring the resources you modeled. When it's done, the screen will look like this:

![Applied Changes](/tutorial-img/03-deploy_containerized_app/applied_changes.png)

Three things to notice about the screen in front of you:
* The name of the Change Set in the top left has changed to `head`. This means you are looking at the current desired state of the model, overlayed with the current state of your resources in AWS.

 * The `Changes Panel` now shows only Applied Changes, with each action we approved earlier now taking place.

* Looking at both the Canvas and the `Diagram Outline Panel`, the red tools 'Changes' notifications have now turned green. The Resources you modeled now exist in AWS, so there are no proposed Changes remaining.

You can inspect the data about your created Resources by selecting the relevant Asset and clicking the on the Resource tab of the `Selected Assets Panel`. Try it with your `EC2 Instance`: open the Resource tab, and notice that once AWS has provisioned your `EC2 Instance` the State will switch to running.

![Running state on Ec2 Instance](/tutorial-img/03-deploy_containerized_app/running_state_on_ec2_instance.png)

### Checking out your new website

To check and see if cat lovers can adopt some cats, you'll need the public IP address of your `EC2 Instance`.

Select your `EC2 Instance` and look in the `Selected Assets Panel` in the Resource tab, and find the `PublicIpAddress` in the JSON output.

![Finding the public IP Address](/tutorial-img/03-deploy_containerized_app/finding_the_public_ip_address.png)

Open a new tab in your web browser, and navigate to the address over http (no SSL for these kitties!). You should see the new website for Whiskers R We!

![Whiskers R We](/tutorial-img/03-deploy_containerized_app/whiskers_r_we.png)

Congratulations! Cats everywhere thank you.
