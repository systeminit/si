---
title: Cleaning up
---

## Cleaning up

While you’re probably very pleased about Whiskers R We coming online, it does cost you a miniscule amount of money. You can reduce your feline financial load by having System Initiative clean up for you.

Use the `Diagram Outline Panel` to select your Assets and delete them. First, ‘Shift-Select’ all the Assets underneath the Region (but not the region itself):

<img src="/tutorial-img/04-cleanup/diagram_outline.png" alt="Diagram Outline" width="50%" height="50%"/> 

Then hit the 'Delete' key on your keyboard, and a modal dialog
will appear, confirming that you want to delete these Assets. Click the ‘Confirm’ button.

<img src="/tutorial-img/04-cleanup/confirm_delete.png" alt="Confirm delete" width="50%" height="50%"/> 

Because this action represents a new change to the read-only `head` version of your Model, a new Change Set is created. The progress bar will update, marking your Assets for deletion. 

The Assets will not disappear from the Workspace - instead, they will be marked with a red X, and any connections they have to undeleted items will be turned into dashed red lines, so you can change your mind and see what has been removed from this Change Set. Your Workspace will look like this:

![Partial Delete](/tutorial-img/04-cleanup/partial_delete.png)

If you want to restore an Asset you might have accidentally deleted, you can select it, and click `Restore Component` in the `Selected Assets Panel`.   

<img src="/tutorial-img/04-cleanup/restore_option.png" alt="Restore option" width="50%" height="50%"/> 

For now, finish cleaning up your Workspace. Select the `Region` Frame, and hit the 'Delete' Key (or right click on the Frame and select `Delete Frame "us-east-2"`) and press the ‘Confirm’ button.

![Right click to delete](/tutorial-img/04-cleanup/right_click_to_delete.png)

Then ‘Shift-Select’ both the Docker Image and the Butane configuration and delete them both. You should now have a Workspace filled with deleted Assets. 

Just like before, if you expand the `Changes Panel` you'll see the full list of proposed changes that you've made. 

While System Initiative always suggests fixes in an order that allows them to be applied in bulk, it never forces you to commit to any actions that would impact Resources directly. You always have full control over the timing and the order of actions. It’s never all-or-nothing. When reviewing the proposed changes, you can toggle and reorder the those changes to control if/when they are applied.

![Final Deletes](/tutorial-img/04-cleanup/final_deletes.png)

For now, since we're cleaning up, keep all of those changes toggled on, and hit the Apply Changes button to merge your changes to `head` - deleting those Assets from the Model and destroying the Resources in AWS.

You’ll see the same wipe and confetti (yay!) and can see your newly merged changes reflected in `head`. The progress bar will update. 

Note: deleting the Security Group will occasionally fail, as the EC2 Instance that is using it has not fully terminated yet. If this happens, just apply the recommendation again. Once it is deleted, you will be back to an empty Workspace:
![Empty Workspace](/tutorial-img/04-cleanup/empty_workspace.png)

### Congratulations!

Congratulations! You have successfully deployed a containerized web application to AWS EC2 with System Initiative - and cleaned up after yourself. :) You learned how:

* All work in System Initiative happens in a Workspaces, which are like instances of System Initiative
* System Initiative ‘models’ the infrastructure and applications you want to see in your Workspace and then tracks the ‘Resources’ that map to them
* You can have multiple versions of the Model at once via Change Sets
* You can construct your Model visually by choosing Assets. 
* Assets have Attributes which map closely to the domain they model
* Assets have Relationships with each other
* System Initiative infers the configuration of your Assets through the Asset's attributes *and* via the Asset’s relationships
* Changing a single attribute will update all related Assets
* Qualifications on your Assets provide real-time feedback on the viability of your Model’s configuration
* Merging a Change Set makes the Model in the Workspace the current ‘head’ Model
* The Model compares to the real-world state of the Resources via Confirmations
* Confirmations make recommendations about what changes you should make, to make the outside world reflect what you have modeled
* You can apply those Proposed Changes all at once, and System Initiative will determine the correct order. 
* System Initiative tracks the created Resource information alongside the attributes of your Model, so you can see them side-by-side
* You can analyze your existing Resources, including refreshing the Resource information in real-time
* When you delete Assets in the Model, System Initiative marks the Asset for deletion but does nothing to the real-world Resource until you decide to `Apply Changes`
* You can delete the Resources all at once or in any order (or time) you choose

We truly appreciate you taking the time to test drive System Initiative. Your next step is to complete a brief survey about your experience while it’s still fresh in your mind. Then return to this tutorial and learn how to customize System Initiative for your specific needs.  