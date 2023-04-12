---
title: Cleaning up
---

## Cleaning up

While you’re probably very pleased about Whiskers R We coming online, it does cost you a miniscule amount of money. You
can reduce your feline financial load by having System Initiative clean up for you.

Start by returning to the Model screen, by clicking the
Model icon <img src="tutorial-img/04-cleanup/model_icon.png" alt="model icon" class="inline" /> in the top navigation bar. You will be
prompted to create a new Change Set (since you merged the previous one!). Accept the default name by clicking on Create
Change Set. Your screen will look like this:

![About to delete Workspace](/tutorial-img/04-cleanup/about_to_delete_workspace.png)

Use the Diagram Outline to select your assets and delete them. First, click the ‘Diagram Outline’ tab in the left panel,
which will show you a list of all the assets in your workspace:

![Diagram Outline](/tutorial-img/04-cleanup/diagram_outline.png)

You should ‘Shift-Select’ all the components underneath the Region (but not the region itself):

The screen should look like this:

Then right-click on one of the selected assets in the Diagram Outliner and click ‘Delete 5 Components’. A modal dialog
will appear, confirming that you want to delete these components. Click the ‘Confirm’ button.

![Confirm delete](/tutorial-img/04-cleanup/confirm_delete.png)

The progress bar will update, marking your assets for deletion. The assets will not disappear from the canvas - instead,
they will be marked with a red X, and any connections they have to undeleted items will be turned into dashed red lines.
This allows you to restore any asset you might have accidentally deleted and see what has been removed from this Change
Set. Your workspace will look like this:

![Partial Delete](/tutorial-img/04-cleanup/partial_delete.png)

Selecting a deleted asset will update the details panel with information about the deleted asset and allow you to
restore it.

![Restore option](/tutorial-img/04-cleanup/restore_option.png)

For now, finish cleaning up your assets. Right-click on the Region frame, and click ‘Delete Frame "us-east-2"’:

![Right click to delete](/tutorial-img/04-cleanup/right_click_to_delete.png)

Confirm the deletion. Then ‘Shift-Select’ both the Docker Image and the Butane configuration on the canvas, and
right-click to delete them both. You should now have a canvas filled with deleted assets:

![Final Deletes](/tutorial-img/04-cleanup/final_deletes.png)

Now you can hit the `Merge` button in the left panel, making this (currently empty) model the version of the world you
want to see. You’ll see the same wipe and confetti (yay!) and redirect to the Apply screen. All of the assets without
resources have been automatically removed from the canvas, while the remaining assets have a trash can over them (to
represent that we intend for them to be deleted, but have not yet done so):

![Initial Delete Fix Screen](/tutorial-img/04-cleanup/initial_delete_fix_screen.png)

The remaining four assets all have failing Confirmations. Select the EC2 Instance and open the confirmation panel to
examine why:

![Why failing confirmations](/tutorial-img/04-cleanup/why_failing_confirmations.png)

Each asset has a similar confirmation - the model has been marked for deletion. Therefore, System Initiative recommends
you delete the corresponding resources in AWS. Rather than use the ‘Select All’ method as you did before, let’s delete
only your EC2 Instance. Start by selecting the checkbox next to the recommendation ‘Delete EC2 Instance’ in the left
panel:

![Delete recommendations](/tutorial-img/04-cleanup/delete_recommendations.png)

Then click the ‘Apply’ button to apply this action to AWS. The progress bar will update, and the apply history will
eventually indicate that you successfully deleted your EC2 Instance. The EC2 Instance on the canvas will disappear,
leaving a screen that looks like this:

![Deleting one thing](/tutorial-img/04-cleanup/deleting_one_thing.png)

While System Initiative always suggests fixes in an order that allows them to be applied in bulk, it never forces you to
commit to any actions that would impact resources directly. You always have full control over the timing and the order
of actions. It’s never all-or-nothing.

Check the ‘Select All’ box in the left panel, and then apply your changes to delete your remaining Ingress, Key Pair,
and Security Group from AWS.

Note: deleting the Security Group will occasionally fail, as the EC2 Instance that is using it has not fully terminated
yet. If this happens, you can try to apply the recommendation again. Once it is deleted, you will be back to an empty
workspace:

![Empty Workspace](/tutorial-img/04-cleanup/empty_workspace.png)

### Congratulations!

Congratulations! You have successfully deployed a containerized web application to AWS EC2 with System Initiative - and
cleaned up after yourself. :) You learned how:

* All work in System Initiative happens in Workspaces, which are like instances of System Initiative
* System Initiative ‘models’ the infrastructure and applications you want to see in your workspace and then tracks the
  ‘resources’ that map to them
* You can have multiple versions of the model at once via Change Sets
* You can construct your model visually by choosing assets
* Assets have Attributes which map closely to the domain they model
* Assets have Relationships with each other
* System Initiative infers the configuration of your assets through the assets attributes *and* via the asset’s
  relationships
* Changing a single attribute will update all related assets
* Qualifications on your assets provide real-time feedback on the viability of your model’s configuration
* Merging a Change Set makes the model it has the current ‘HEAD’ model
* The HEAD model compares to the real-world state of the resources via Confirmations
* Confirmations make recommendations about what actions you should take to make the outside world reflect what you have
  modeled
* You can apply those recommendations all at once, and System Initiative will determine the correct order
* System Initiative tracks the created resource information alongside the attributes of your model, so you can see them
  side-by-side
* You can analyze your existing resources, including refreshing the resource information in real-time
* When you delete things in the model, System Initiative marks the asset for deletion but does nothing to the real-world
  resource
* Once deleted assets confirmations run, System Initiative makes a recommendation to delete resources
* You can delete the resources all at once or in any order (or time) you choose

We truly appreciate you taking the time to test drive System Initiative. Your next step is to complete a brief survey
about your experience while it’s still fresh in your mind. Then return to this tutorial and learn how to customize
System Initiative for your specific needs.  