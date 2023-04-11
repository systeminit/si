---
title: Deploy a containerized web application to an AWS EC2 Instance
---

## Deploy a containerized web application to an AWS EC2 instance

In this section, you will learn how to deploy a simple containerized web application to an AWS EC2 Instance
running [Fedora CoreOS](https://getfedora.org/en/coreos/download?tab=cloud_launchable&stream=stable&arch=x86_64). You
will build a theoretical model of your infrastructure that you will later compare to the real world to create the
necessary resources.

At this point, you should have this tutorial open in one browser window and your System Initiative dev workspace open in
another. Your System Initiative workspace will look like this:

![Initial System Initiative Screen](tutorial-img/03-deploy_containerized_app/initial_system_initiative_screen.png)

All work in System Initiative takes place in a workspace - think of it like an 'instance' of System Initiative. In the
future, you will be able to create workspaces that run locally, in a datacenter, in your cloud provider, or fully
managed by System Initiative.

### Models and Resources

In System Initiative, you build a 'model' of your infrastructure and applications. Think of this model as a simulation
of what you want to see in the real world. It tries to mirror the capabilities of the real-world 1:1 while also
providing immediate feedback and inferring configuration on your behalf. When working with the model, you can be
confident that nothing changes in the real world - it is purely a hypothesis about what you believe should exist.

When you decide that the model reflects the reality you want, System Initiative will compare it to the resources you
have in the world. If the real-world resources don't match the model, System Initiative will suggest appropriate actions
to take – for example, creating an EC2 instance or a security group – then track information about the resource itself,
allowing you to view both the current state of the resource and your model's attributes side-by-side.

### Creating a Change Set

When you want to change your model, you do so inside a 'Change Set'. Change Sets allow you to create new 'versions' of
your model without impacting the version of your model that reflects reality. Think of it like a lightweight git branch
that auto-rebases on main - where main is the version that you believe best reflects the reality you want. We call the
'main' version of your model the 'HEAD'.

If you've been following along, your System Initiative workspace is prompting you to create a new Change Set:

![Create a Change Set](tutorial-img/03-deploy_containerized_app/create_a_change_set.png)

You are in development mode, so we automatically suggest a Change Set name for you. Click the '+ Create change set'
button to create your new 'Demo 1' Change Set.

### Modeling your infrastructure

With your Change Set created, System Initiative should now look like this:

![Empty System Initiative Workspace](tutorial-img/03-deploy_containerized_app/empty_system_initiative_workspace.png)

The primary navigation bar at the top has the Model icon selected 
<img src="tutorial-img/03-deploy_containerized_app/model_icon.png" alt="Model Icon" class="inline" />
indicating that you want to work on your 
infrastructure model. The left side of the screen contains the Asset Palette - a collection of various Assets we can use
to build our model. In the center of the screen is a canvas where you will design your model, with a progress bar on
top. To the right is a details panel that will show you the selected asset's attributes, source code, and resources. At
the bottom is a Status Bar showing you information and feedback about the model.

Our application is a simple web application
for [adopting cats](https://www.youtube.com/results?search_query=whiskers+r+we), which is in
a [public docker container on Docker Hub](https://hub.docker.com/r/systeminit/whiskers). Click on the 'Docker Image'
asset in the Asset Palette (you may need to scroll down), then click again to place it on the canvas.

You will see the progress bar update, indicating that System Initiative is updating the model of your infrastructure.
When it finishes, click on the asset you placed on the canvas, and your workspace will look something like this:

![Workspace with Docker Image](tutorial-img/03-deploy_containerized_app/workspace_with_docker_image.png)

Let's take a minute to orient you to all the information we're showing you. The first thing to look at is the Docker
Image asset on the canvas:

![Docker image component](tutorial-img/03-deploy_containerized_app/docker_image_component.png)

The asset has an icon indicating its family (in this case, the docker logo). Then a randomly generated name: in our
case, si-6865. Below that is the 'type' of this asset - a Docker Image. To the right is the green 'plus' icon - this is
showing you that this asset was added to the model in this Change Set. Below that are two Output Sockets - one named
'Container Image' and the other named 'Exposed Ports' (we will go into more detail about these later.) Finally, you see
a circular red X, which represents the Qualification status of the asset.

Qualifications tell you if the asset meets all the requirements to function in the real world (whether it is 'qualified
for use'). They are like built-in, real-time tests for your model. Every time we change anything about our asset, we
will recheck all the qualifications for that asset. The red X means your Docker Image is not qualified. To find out why,
use the Qualifications tab on the status bar:

![Qualifications tab on the status bar](tutorial-img/03-deploy_containerized_app/qualifications_tab_on_the_status_bar.png)

Click the Qualifications tab to expand the status bar:

![Qualifications expanded](tutorial-img/03-deploy_containerized_app/qualifications_expanded.png)

You can see that your Docker Image is not qualified because your registry has no docker image named 'si-6865'. To fix
it, we need to configure the attributes of our Docker Image to point at a valid docker image.

To do that, we will use the details panel on the right side of the screen. Yours should look something like this:

![Docker image details panel](tutorial-img/03-deploy_containerized_app/docker_image_details_panel.png)

The attributes tab shows the currently selected asset at the top and its current state within the Change Set (in our
case, added). Below that is a short history of the last change made to this asset. Beneath that are tabs you can toggle
between for various detailed views: Attributes, Code, and Resources. The Attributes tab shows the configurable
attributes of this asset, divided into two sections - `si` attributes, which are helpful in changing how `si` interprets
your configuration, and `domain` attributes, which model the upstream's configuration as closely as possible. The Code
tab will show any generated source code, if necessary. Once the asset attaches to a real-world resource, you can view
that data in the Resource tab.

The container publishes to Docker Hub as `systeminit/whiskers`. Change the `si/name` of the Docker Image asset
to `systeminit/whiskers` now. Press the 'enter' key when you are done. System Initiative will automatically save any
data you enter into the attributes panel. You'll see the progress bar update, and shortly after that the qualification
icon will turn green, both on the canvas and in the qualifications status bar. Once everything is green, collapse the
status bar by clicking the down arrow on the right-hand side of the bar.

![Whiskers docker image workspace](tutorial-img/03-deploy_containerized_app/whiskers_docker_image_workspace.png)

Notice that, in addition to setting the `si/name` attribute, this also sets `domain/image` to the same value. In System
Initiative, assets can infer configuration - either from their attributes or from relationships they have with other
assets. It's a powerful way to easily generate a correct configuration.

Our Docker Image exposes a web server running on port 80. Let's add that to our Docker Image's attributes now. Click on
the `+ Add to array` button in the `domain/ExposedPorts[0]` attribute:

![Add to array](tutorial-img/03-deploy_containerized_app/add_to_array.png)

The progress bar will update, and you will see a new entry for the ExposedPort:

![Added ExposedPort](tutorial-img/03-deploy_containerized_app/added_exposedport.png)

Put the value `80/tcp` in this field, and hit the `enter` key to save. The `80/tcp` syntax comes directly from Docker
itself - like the image name, System Initiative maps the upstream behavior 1:1 - this allows you to easily transfer your
existing knowledge about a given technology into System Initiative. Your attributes panel should now look like this:

![Whiskers docker attribute panel](tutorial-img/03-deploy_containerized_app/whiskers_docker_attribute_panel.png)

We want you to deploy your Docker Image on a [Fedora CoreOS](https://getfedora.org/en/coreos) system, in part because it
has an excellent method for generating configuration for cloud instances
called [Butane](https://coreos.github.io/butane/getting-started/).

You can use the Docker Image we just configured to automatically generate a Butane configuration for Fedora: click the
'CoreOS Butane' asset in the Asset Palette, then click on the canvas, placing it to the right of your Docker Image.
Notice that your new Butane asset has a socket on the left side - this is an `Input Socket`.

Then connect the `Container Image` output socket of our Docker Image to the matching `Container Image` input socket on
your new Butane asset by clicking on the output socket and dragging the line to the input socket.

![Connected sockets](tutorial-img/03-deploy_containerized_app/connected_sockets.png)

When the progress bar has finished, click the Butane asset to load its details. You can see at a glance that it is
already qualified (the circular green checkmark), indicating that System Initiative believes this is a valid
configuration. Scroll down to the `domain/systemd/units/[0](unit)` attribute (you might need to expand the size of the
text area to read it well). It should look something like this:

![Butane details](tutorial-img/03-deploy_containerized_app/butane_details.png)

System Initiative used the relationship between the Docker Image and the Butane instance to automatically infer that you
wanted to create a [systemd unit file](https://www.freedesktop.org/software/systemd/man/systemd.unit.html), and wrote it
for you - including translating the exposed ports to publish commands to [podman](https://podman.io/)!

Butane processes its configuration into a json format known as [Ignition]( https://coreos.github.io/ignition/examples/).
System Initiative automatically generates the Ignition data for you based on the provided Butane configuration. Select
the `Code` tab in the details panel to see it.

![Code View](tutorial-img/03-deploy_containerized_app/code_view.png)

It's time to start modeling the deployment of our CoreOS instance to AWS. First, you'll need to pick a Region for our
deployment. Select the `AWS Region` asset from the Asset Palette, and drop it on the canvas to the right of our Butane
configuration.

![Workspace with region](tutorial-img/03-deploy_containerized_app/workspace_with_region.png)

Notice that this asset looks different from the previous two! They were `components`, while AWS Region is a `frame`.
Frames are a way to organize components on the diagram while configuring their contents, or aggregate components of the
same type together for easier relationship creation. The Region frame is a _configuration_ frame, meaning that any
component placed inside is automatically configured by it.

Resize the Region to be a little larger by clicking and dragging the corner of the frame. You can see in the lower right
corner of the frame that it is not qualified - that's because you haven't decided which AWS Region to use. Select the
Region, click the Attributes tab on the details panel, and then set the `domain/region` attribute to `us-east-2`. When
the progress bar finishes, you will see that our qualification for this region turns from a red X to a green check -
you're good to go. System Initiative has also helpfully inferred the `si/name` of the Region for you!

![Set the region](tutorial-img/03-deploy_containerized_app/set_the_region.png)

The application runs on an EC2 Instance, so let's start there. Select the `AWS EC2 Instance` asset from the Asset
Palette, and click inside the Region on the canvas. You can then drag the EC2 Instance into the upper right corner of
the Region frame by clicking and dragging on it. You can see that the Region input socket on the EC2 Instance is already
filled - from the Region frame itself. Things should look like this:

![Ec2 Instance in Region Frame](tutorial-img/03-deploy_containerized_app/ec2_instance_in_region_frame.png)

Let's work backward, connecting all the things our EC2 Instance needs in order to function. Your Butane configuration
has a matching `User Data` output socket to your new EC2 Instance's `User Data` input socket. Let's connect them now:

![Connected Inside a Region](tutorial-img/03-deploy_containerized_app/connected_inside_a_region.png)

With the EC2 Instance selected, you can check that the user data has been populated by looking at the `domain/UserData`
field in the attributes panel or the generated JSON code in the Code panel.

Walking the input sockets of our EC2 Instance from top to bottom, you can see that the next thing you will need is a
Security Group ID. Add a Security Group into the Region and connect it to our EC2 Instance. I like to place the Security
Group in the lower left corner of the Region frame.

A cat adoption website that nobody can reach from the outside world would be a sad, lonely website. To fix that, you
need to add an Ingress rule to our Security Group. Select the `AWS Ingress` asset from the asset pallet, and place it to
the right of the Security Group in the Region frame. Then connect the Security Group to the Ingress rule.

Notice that the Ingress component takes an input socket named `Exposed Ports` - and suspiciously, your Docker Image has
an output socket with that name. Connect the Docker Image to the Ingress rule as well! Then select the Ingress rule, and
switch to the Code view in the details panel.

![Connect a Security Group and Ingress Rule](tutorial-img/03-deploy_containerized_app/connect_a_security_group_and_ingress_rule.png)

Check out how System Initiative has written the `IpPermissions` automatically. You now have a single configuration
attribute, the ExposedPorts of our Docker Image, automatically configuring the Operating System, EC2 User Data, and our
Ingress rule. Select your Docker Image, go to the Attribute tab in the details panel, and change the ExposedPort
to `8080/tcp` rather than `80/tcp`. You will see System Initiative calculate the scope of the update, then show that it
is updating the configuration of all 3 impacted assets on the canvas!

That was fun, but let's switch it back to port 80. Select the Docker Image, go to the Attribute tab in the details
panel, and change ExposedPort back to 80/tcp.

Your Ingress rule has a Warning on its configuration. Open the Qualification panel in the status bar, select the Ingress
rule on the canvas, and you can see why:

![Warning on Ingress](tutorial-img/03-deploy_containerized_app/warning_on_ingress.png)

The configuration for this Ingress rule isn't technically valid (yet!) - for it to work, you need to set the Security
Group ID. You don't have one of those yet because everything done on the model screen happens in a Change Set and
doesn't impact the outside world. Since your Ingress rule connects to your Security Group, which will provide the ID
eventually, the warning will resolve itself just fine later.

Returning to your EC2 Instance's input sockets, the next socket is Image ID. In AWS EC2, you use an AMIs ID as the Image
ID. Click the `AWS AMI` asset in the Asset Palette, then place it within the Region on the canvas. Connect the Image ID
output socket on the AMI to the EC2 Instances Image ID input socket. Then, with the AMI selected, go to the Attributes
tab in the details panel and populate `domain/ImageId`
with `ami-0ed17ac79c5602c98` ([or the latest AMI for us-east-2 available on the CoreOS Download page](https://getfedora.org/en/coreos/download?tab=cloud_launchable&stream=stable&arch=x86_64)).

![AMI joins the party](tutorial-img/03-deploy_containerized_app/ami_joins_the_party.png)

There is one more open input socket, the Key Name. AWS EC2 uses SSH Key Pairs to authenticate to the instances you boot.
Add a Key Pair asset to your region, and connect it to your EC2 Instance.

![Key Pair joins the party](tutorial-img/03-deploy_containerized_app/key_pair_joins_the_party.png)

You are down to a single qualification error on your model. Select the EC2 Instance, and you can see that the issue is
that we have not yet configured the `/domain/InstanceType` attribute.

![Needs an Instance Type](tutorial-img/03-deploy_containerized_app/needs_an_instance_type.png)

Go to the `domain/InstanceType` attribute in the details panel, and set it to `t3.micro`. You should see the 'All Fields Are
Valid' qualification turn green and the remaining warnings are related to the Key Pair and Security Group not existing
yet (which is fine - it will soon enough!) Close the status bar, and let's get this show on the road.

### Merge your Change Set

In System Initiative, like in life, you can imagine as many potential realities as you would like - but in the end, you
have to pick which one to manifest into the universe. This model looks good - all the qualifications are green (or are
expected warnings). Let's ship it.

To make this model the current model you want to see in the world, click the `Merge` button in the upper-left corner,
right above the Asset Palette.

![Merge button](tutorial-img/03-deploy_containerized_app/merge_button.png)

You will see a screen wipe, followed by a fun celebratory confetti cannon, and then you will redirect to the `Apply` <img src="tutorial-img/03-deploy_containerized_app/apply_icon.png" alt="apply icon" class="inline" /> interface.

### Creating your resources in AWS

The apply screen is where we take your current model and compare it to the configuration of the outside world. The
progress bar starts updating, and the assets on the diagram will have a spinner that indicates System Initiative is
evaluating them against AWS. When the progress bar and spinners are complete, the screen will look like this:

![Apply Screen](tutorial-img/03-deploy_containerized_app/apply_screen.png)

You might notice that the interface has changed. The left panel is now showing you a list of recommended actions to take
to keep your model in sync with AWS. The center canvas is essentially the same but now shows a 'frozen' version of your
HEAD model, along with more information about how it compares to the world. The right panel now shows the history of any
recommendations you apply to the world. Finally, the status bar has added two new panels - Apply History and
Confirmations.

Looking at the diagram, some of our assets now have an extra warning icon to the right of the qualification status -
this shows their Confirmation status.

![Ec2 Apply Closeup](tutorial-img/03-deploy_containerized_app/ec2_apply_closeup.png)

A Confirmation is a check that takes the model you have configured and compares it to what is known about the resources
you have. Unlike a Qualification, Confirmations also can recommend **actions** for you to take in order to bring reality
in line with your model. If you select your EC2 Instance on the Canvas, and open the Confirmations panel in the status
bar, you will see two confirmations:

![Confirmations Status](tutorial-img/03-deploy_containerized_app/confirmations_status.png)

The failing confirmation is checking to see if the EC2 Instance exists - and since it does not, it recommends the
'Create EC2 Instance' action. You can see that in the Recommendations section in the left panel:

![Recommendations close up](tutorial-img/03-deploy_containerized_app/recommendations_close_up.png)

Each of the four AWS assets you used do not exist, and will therefore have a similar recommended action - to create
them. To do that, click the box next to 'Select All', and hit the 'Apply' button.

The progress bar will update with the list of applying actions, and the recommendations will flow from the
recommendations list to the 'Apply History' panel on the right.

![Applied fixes to workspace](tutorial-img/03-deploy_containerized_app/applied_fixes_to_workspace.png)

You can see that all the recommended fixes succeeded and that they were applied by you. The confirmations that were red
have now turned green - your model now matches the resources we expected in AWS. Go ahead and close that sub-panel now
by clicking on the down arrow on the bar.

You can inspect the data about your created resources by clicking on the chevrons in the Apply History. For example,
look at the resource properties that were received as a result of creating your new EC2 Instance:

![Inspect Pending EC2 Instance in fix output](tutorial-img/03-deploy_containerized_app/inspect_pending_ec2_instance_in_fix_output.png)

Of particular interest is the fact that your EC2 Instance has a 'State' of 'pending' - that means that it hasn't yet
finished being created in AWS. In order to see if you successfully started your application, you will need to `Refresh`
the resources data to get the Public IP Address.

### Checking out our new website

System Initiative keeps track of both the theoretical model you want to see and
your real-world resources. When you want to explore how things are right now,
you do that from the Analyze page. Click the Eyeball 
<img src="tutorial-img/03-deploy_containerized_app/eyeball.png" width="60" alt="Eyeball" class="inline" />
 in the top navigation to move from the Apply screen to the Analyze screen.

![Analyze screen workspace view](tutorial-img/03-deploy_containerized_app/analyze_screen_workspace_view.png)

On the left, you have the `Diagram Outline` view, which has a list of all the assets in your workspace, and a recap of
their current status. The center is now a read-only view of the canvas. The right hand panel is a read-only version of
the details panel from the Model screen you used earlier. The status bar has the same information it has on the Apply
screen.

To check and see if cat lovers can adopt some cats, you'll need the public IP address of your EC2 Instance. Select your
EC2 Instance from the Diagram Outline on the left, and you should see the following:

![Analyze screen Ec2 Instance Selected](tutorial-img/03-deploy_containerized_app/analyze_screen_ec2_instance_selected.png)

Click the Refresh button <img src="tutorial-img/03-deploy_containerized_app/refresh_button.png" alt="refresh button" class="inline"> in the details panel. You will see it spin and the resource information below update. Depending on how
long AWS takes to provision your EC2 Instance, you may need to hit this button multiple times. Eventually, you will see
the State switch to running:

![Running state on Ec2 Instance](tutorial-img/03-deploy_containerized_app/running_state_on_ec2_instance.png)

Once this has happened, you can find the `PublicIpAddress` in the JSON output:

![Finding the public IP Address](tutorial-img/03-deploy_containerized_app/finding_the_public_ip_address.png)

Open a new tab in your web browser, and navigate to it over http (no SSL for these kitties!). You should see the new
website for Whiskers R We!

![Whiskers R We](tutorial-img/03-deploy_containerized_app/whiskers_r_we.png)

Congratulations! Cat's everywhere thank you.
