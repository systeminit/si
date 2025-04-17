<template>
  <section class="grid gap-md h-[100%]">
    <div>Name</div>
    <div class="row-span-2">Qualifications</div>
    <div class="row-span-4 grid grid-cols-subgrid gap-sm">
      <div>Documentation</div>
      <div>Actions</div>
      <div>Connections</div>
    </div>
    <div class="row-span-2 scrollable">
      <p>
        Lorem ipsum dolor sit amet, consectetur adipiscing elit. Integer id
        mauris libero. Donec maximus rutrum ipsum accumsan porttitor. Nulla
        convallis leo ac turpis volutpat tempor. Maecenas sollicitudin, magna
        vitae blandit varius, sapien diam molestie ex, id feugiat ligula mi id
        ipsum. Cras et vehicula urna, vitae ultrices sapien. Maecenas vel massa
        id ligula luctus pharetra feugiat in erat. Aliquam tortor orci,
        hendrerit vitae massa at, tristique euismod mauris. In egestas
        pellentesque mauris, in commodo urna pellentesque et.
      </p>
      <p>
        Cras eleifend libero sapien, sit amet mattis nibh viverra sit amet. Sed
        imperdiet nec est sit amet elementum. Proin scelerisque dictum
        vulputate. Sed cursus accumsan ipsum sit amet hendrerit. Sed facilisis
        quam ac sapien convallis, in interdum nibh mattis. Integer in ligula
        erat. Curabitur nisl orci, semper ut varius vel, pellentesque vel
        tellus. Sed id metus ut ipsum cursus lobortis tincidunt a turpis.
        Phasellus nulla quam, blandit eget sapien et, sagittis ullamcorper enim.
        Integer eget ex tincidunt, volutpat nisl et, ullamcorper augue.
        Suspendisse et sem quis est facilisis suscipit.
      </p>
      <p>
        Morbi auctor egestas erat, eu venenatis erat hendrerit eu. Morbi ut
        mollis nibh. Aliquam justo tellus, fermentum volutpat dolor et, rutrum
        commodo dui. Nullam sed libero in lacus fermentum hendrerit eu sed urna.
        Aenean non commodo diam. Aliquam porttitor odio in ex placerat sagittis.
        Proin gravida mollis tellus sed consequat. Maecenas ipsum orci, feugiat
        at arcu sed, tincidunt accumsan sapien.
      </p>
      <p>
        Sed elementum eu magna convallis facilisis. Donec ullamcorper
        condimentum justo. Vestibulum vestibulum ex at mauris tincidunt, id
        tempor lorem sollicitudin. Duis et fermentum nisl. Nam lacinia elementum
        quam, ut scelerisque leo euismod eget. Duis luctus ligula eget tempor
        pulvinar. Quisque scelerisque, lorem ut malesuada hendrerit, neque nulla
        imperdiet enim, vitae ultricies dolor augue sed leo. Nam vel nunc
        tortor. Pellentesque enim ante, pharetra at risus id, varius dictum
        turpis. Fusce sit amet erat nec mauris venenatis interdum vel vel ipsum.
        Pellentesque vehicula quam tellus, a pellentesque nunc pretium quis.
      </p>
    </div>
    <div>Code / Diff / Debug</div>
    <div class="col-span-2">Footer</div>
  </section>
</template>

<script lang="ts" setup></script>

<style lang="css" scoped>
section.grid {
  grid-template-columns: 50% 20% 30%;
  grid-template-rows: 4rem 1fr 1fr 2rem;
}
</style>
