---
title: Introduction
hideWorkspaceLink: true
---

## Welcome to System Initiative!

We're so excited to have you as part of this invite-only preview of what we've been up to for the last while. The whole thing will be open sourced and we'll have a wider public release soon, but blah blah blah...

### FrieNDA
This is an invite-only preview release,
intended to let you experience System Initiative and for us to shake out
the (probably many) issues you might have. We ask that you not speak
publicly about your use (positively or negatively), share screenshots,
etc. There are private channels we will invite you to in Discord where you
will be able to discuss System Initiative with us, and with other
participants in the preview. We're not asking you to sign an NDA - but we
are relying on your good-will and judgement as a friend of System Initiative, Inc.

**Things you should do:**
- <Icon name="check-circle"></Icon>Talk with us privately on discord
- <Icon name="check-circle"></Icon>Tell us what you love and 
hate about the product
- <Icon name="check-circle"></Icon>Send a pull-request to change something

**Things you should NOT do:**
- <Icon name="x-circle"></Icon>Post screenshots of the product on twitter
- <Icon name="x-circle"></Icon>Tell folks in public places what you love / hate



![Image 1](/tutorial-img/img1.png)


I'm baby williamsburg tote bag fingerstache jianbing art party coloring book, yuccie hoodie artisan organic shoreditch air plant gluten-free iceland plaid. Glossier meditation woke disrupt, cupping forage blue bottle viral distillery. Occupy woke aesthetic, hell of roof party gastropub knausgaard banjo affogato church-key street art skateboard. Semiotics live-edge pitchfork vibecession man bun, chillwave gentrify blackbird spyplane farm-to-table hell of marfa.

```js
async function qualificationDockerImageExists(component) {
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


and some `executeFn(foo, bar, "asdf", 1)` here too for fun.


Kombucha jawn shoreditch listicle wayfarers hell of vaporware, godard meh sriracha jianbing cred williamsburg. Aesthetic iPhone hammock, mukbang venmo neutra praxis +1 90's occupy pop-up fam raclette deep v. Seitan tilde PBR&B, hammock DIY art party vibecession adaptogen authentic pork belly vegan freegan polaroid offal. Succulents vinyl unicorn fit readymade bitters marfa vice salvia. Whatever paleo marxism shabby chic food truck pug, viral taiyaki.

Paleo blog pitchfork grailed, lo-fi fam irony mixtape 90's cold-pressed polaroid. Gastropub raclette chillwave celiac, hexagon migas everyday carry live-edge pickled hammock. Jean shorts green juice yes plz, semiotics irony franzen taiyaki. Sartorial flannel cardigan, 90's cliche locavore squid vaporware schlitz deep v vexillologist paleo hoodie. Live-edge salvia portland ugh, activated charcoal mlkshk food truck poutine.

Offal trust fund gluten-free DSA, kitsch beard four loko typewriter. Seitan messenger bag hella, mukbang cupping yuccie retro. Scenester occupy poutine, food truck irony 3 wolf moon jianbing. Tonx wolf leggings roof party selvage. Hoodie banjo glossier, pabst swag JOMO bicycle rights sartorial tumeric leggings bushwick cardigan readymade. Keytar williamsburg coloring book, YOLO woke cold-pressed succulents leggings skateboard etsy. Fanny pack letterpress hella live-edge, enamel pin same banh mi gentrify +1 retro shaman pickled ramps.

Mukbang hammock thundercats selfies succulents. Synth pop-up twee aesthetic gluten-free. Art party ugh butcher intelligentsia dreamcatcher. Praxis ennui cloud bread vape forage offal, austin shoreditch retro swag mlkshk flexitarian echo park skateboard.
