---
title: Install and run System Initiative
hideWorkspaceLink: true
---
## Install and run System Initiative

Let's do this:
* Visit the <router-link :to="{ name: 'download' }" target="_blank">Install page</router-link> and follow the instructions to install System Initiative.
* Run the command `si start` in a terminal window, which will check for system dependencies, prompt for the necessary credentials, and download/start the System Initiative components.

The button below should have two red beacons, which will turn green and say "Frontend online" and "Backend online" when you're running System Initiative and ready to launch. Click this button to login and open your workspace when it's ready:

<!-- must wrap in a div to undo some of the automatic styling because otherwise it will be put inside a <p> tag -->
<div><workspace-link-widget></workspace-link-widget></div>

If you run into trouble - hit us up on <a href="https://discord.com/channels/955539345538957342/1080953018788364288" target="_blank">Discord</a>, and we’ll get you sorted.
