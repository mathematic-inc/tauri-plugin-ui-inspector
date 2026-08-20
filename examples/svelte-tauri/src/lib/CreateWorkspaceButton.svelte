<script lang="ts">
  interface Props {
    onclick?: () => void;
  }

  let { onclick }: Props = $props();
</script>

<button
  class="primary"
  data-testid="create-workspace"
  data-ui-component="CreateWorkspaceButton"
  {onclick}>Create workspace</button
>

<style>
  .primary {
    width: 184px;
    height: 40px;
    border: 1px solid #38bdf8;
    border-radius: 3px;
    background: #087bea;
    color: #fff;
    font:
      600 16px/1 system-ui,
      sans-serif;
  }

  .primary:hover,
  .primary:focus-visible {
    background: #0b8cff;
    outline: 2px solid #7dd3fc;
    outline-offset: 2px;
  }
</style>
