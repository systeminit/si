---
title: Install and run System Initiative 
hideWorkspaceLink: true
---
## Install and run System Initiative 

Let's do this: 
* Visit the <router-link :to="{ name: 'download' }" target="_blank">Install page</router-link> and follow the instructions to install System Initiative. 
* Run the command `si start` in a terminal window, which will check for system dependencies, prompt for the necessary credentials, and download/start the System Initiative components. To see other common commands, run the command `si help`.

The button below should have two red beacons, which will turn green and say "Frontend online" and "Backend online" when you're running System Initiative and ready to launch. Click this button to login and open your workspace:

<!-- must wrap in a div to undo some of the automatic styling -->
<p class="escape"><workspace-link-widget></workspace-link-widget></p>

If you want to compile and run the developer environment locally, see instructions in our <a href="https://github.com/systeminit/si" target="_blank">GitHub repo</a>. 

If you run into trouble - hit us up on <a href="https://discord.com/channels/955539345538957342/1080953018788364288" target="_blank">Discord</a>, and we’ll get you sorted.
