<template>
  <div class="h-full w-full flex flex-col p-2 lg:p-10">
    <div class="flex flex-row">
      <div>
        <img
          src="~/assets/si-logo-with-text-web-white-medium-sm_icon.svg"
          class="w-48"
        />
      </div>
    </div>
    <div
      class="flex flex-row h-full place-items-center md:place-items-center flex-wrap-reverse"
    >
      <div class="flex flex-col w-full md:w-3/4 md:h-full justify-center">
        <div class="flex">
          <h1 class="font-extrabold text-4xl md:text-6xl">
            Power tools for DevOps teams
            <div class="cursor flex h-8 w-4 md:w-8 md:h-12" />
          </h1>
        </div>
        <div class="flex pt-8 pb-8">
          <!-- Begin Mailchimp Signup Form -->
          <div id="mc_embed_signup" class="flex w-1/2">
            <form
              id="mc-embedded-subscribe-form"
              action="https://systeminit.us2.list-manage.com/subscribe/post?u=8cc24ffff1c789353092c5fc1&amp;id=49f3d02ccb"
              method="post"
              name="mc-embedded-subscribe-form"
              class="validate"
              target="_blank"
              novalidate
            >
              <div id="mc_embed_signup_scroll" class="w-full">
                <div class="mc-field-group w-full">
                  <label for="mce-EMAIL" class="sr-only"
                    >Email Address <span class="asterisk">*</span>
                  </label>
                  <input
                    id="mce-EMAIL"
                    placeholder="Email"
                    type="email"
                    value=""
                    name="EMAIL"
                    class="block bg-zinc-800 text-xs border-0 focus:border-0 w-full"
                  />
                </div>
                <div class="mc-field-group mt-1">
                  <label for="mce-FNAME" class="sr-only">First Name </label>
                  <input
                    id="mce-FNAME"
                    type="text"
                    value=""
                    name="FNAME"
                    placeholder="First name"
                    class="block bg-zinc-800 text-xs border-0 focus:border-0 w-full"
                  />
                </div>
                <div class="mc-field-group mt-1">
                  <label for="mce-LNAME" class="sr-only">Last Name </label>
                  <input
                    id="mce-LNAME"
                    type="text"
                    value=""
                    name="LNAME"
                    class="block bg-zinc-800 text-xs border-0 focus:border-0 w-full"
                    placeholder="Last name"
                  />
                </div>
                <div class="mc-field-group mt-1 flex">
                  <input
                    id="mce-LNAME"
                    type="checkbox"
                    value=""
                    name="LNAME"
                    class="block bg-zinc-800 text-xs border-0 focus:border-0"
                    placeholder="last name"
                  />
                  <label class="block text-xs ml-1" for="mce-LNAME"
                    >I agree to receive marketing emails from System
                    Initiative</label
                  >
                </div>

                <div id="mce-responses" class="clear">
                  <div
                    id="mce-error-response"
                    class="response"
                    style="display: none"
                  ></div>
                  <div
                    id="mce-success-response"
                    class="response"
                    style="display: none"
                  ></div>
                </div>
                <!-- real people should not fill this in and expect good things - do not remove this or risk form bot signups-->
                <div
                  style="position: absolute; left: -5000px"
                  aria-hidden="true"
                >
                  <input
                    type="text"
                    name="b_8cc24ffff1c789353092c5fc1_49f3d02ccb"
                    tabindex="-1"
                    value=""
                  />
                </div>
                <div class="clear mt-1">
                  <input
                    id="mc-embedded-subscribe"
                    class="block bg-emerald-400 text-black font-mono text-xs p-2 border-0 focus:border-0"
                    type="submit"
                    value="REQUEST AN INVITE"
                    name="Subscribe"
                  />
                </div>
              </div>
            </form>
          </div>

          <!--End mc_embed_signup-->
        </div>
      </div>
      <div class="flex md:w-1/4 rotate-90 md:rotate-0 shrink-1 justify-start">
        <img
          class="w-1/2"
          src="~/assets/Circle-01-06-1.png"
          loading="lazy"
          srcset="
            /assets/Circle-01-06-1-p-500.png 500w,
            /assets/Circle-01-06-1.png       594w
          "
          sizes="(max-width: 479px) 79vw, (max-width: 767px) 81vw, 300px"
          alt="circles"
        />
      </div>
    </div>
    <div
      class="flex flex-row border-solid border-t border-zinc-800 pt-10 pb-10 place-content-end pr-10"
    >
      <a href="https://systeminit.pinpointhq.com/">Jobs</a>
    </div>
    <div class="flex flex-row text-zinc-600 place-content-center">
      &copy; 2022, System Initiative, Inc. all rights reserved.
    </div>
  </div>
</template>

<style scoped>
.cursor {
  content: "";
  background: #ffffff;
  display: inline-block;
  animation: cursor-blink 1.5s steps(2) infinite;
}

@keyframes cursor-blink {
  0% {
    opacity: 0;
  }
}
</style>
