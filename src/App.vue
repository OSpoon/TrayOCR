<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core"
import { listen } from "@tauri-apps/api/event"
import { useColorMode } from "@vueuse/core"
import { onBeforeUnmount, onMounted, ref } from "vue"
import { toast } from "vue-sonner"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Toaster } from "@/components/ui/sonner"
import { useUpdater } from "@/composables/useUpdater"
import UpdaterDialog from "@/features/updater/UpdaterDialog.vue"
import "vue-sonner/style.css"

const greetMsg = ref("")
const name = ref("")
const updater = useUpdater()
const colorMode = useColorMode()

let unlistenSetTheme: null | (() => void) = null

onMounted(async () => {
  unlistenSetTheme = await listen<string>("app://set-theme", (event) => {
    const next = event.payload
    if (next === "light" || next === "dark" || next === "auto")
      colorMode.value = next
  })
})

onBeforeUnmount(() => {
  unlistenSetTheme?.()
  unlistenSetTheme = null
})

async function greet() {
  greetMsg.value = await invoke("greet", { name: name.value })
  toast.success("Greeting sent", {
    description: greetMsg.value,
  })
}
</script>

<template>
  <main class="relative min-h-screen w-full">
    <div class="mx-auto flex min-h-screen w-full max-w-3xl flex-col items-center justify-center gap-6 px-4 py-10 text-center">
      <h1 class="text-4xl font-bold tracking-tight">
        Welcome to Tauri + Vue
      </h1>
      <div class="flex flex-wrap items-center justify-center gap-4">
        <a href="https://vite.dev" target="_blank" rel="noreferrer">
          <img src="/vite.svg" class="h-24 p-3 transition hover:drop-shadow-[0_0_1.5em_#747bff]" alt="Vite logo">
        </a>
        <a href="https://tauri.app" target="_blank" rel="noreferrer">
          <img src="/tauri.svg" class="h-24 p-3 transition hover:drop-shadow-[0_0_1.5em_#24c8db]" alt="Tauri logo">
        </a>
        <a href="https://vuejs.org/" target="_blank" rel="noreferrer">
          <img src="./assets/vue.svg" class="h-24 p-3 transition hover:drop-shadow-[0_0_1.5em_#249b73]" alt="Vue logo">
        </a>
      </div>

      <p class="text-muted-foreground">
        Click on the Tauri, Vite, and Vue logos to learn more.
      </p>

      <form class="grid w-full max-w-md grid-cols-1 gap-3 sm:grid-cols-[1fr_auto]" @submit.prevent="greet">
        <Input id="greet-input" v-model="name" placeholder="Enter a name..." />
        <Button type="submit">
          Greet
        </Button>
      </form>

      <p class="min-h-6 text-sm text-foreground">
        {{ greetMsg }}
      </p>
    </div>
  </main>

  <UpdaterDialog
    v-model:open="updater.dialogOpen.value"
    :title="updater.dialogTitle.value"
    :description="updater.dialogDescription.value"
    :progress="updater.progress.value"
    :busy="updater.busy.value"
    :pending-restart="updater.pendingRestart.value"
    @restart="updater.restartApp"
  />

  <Toaster />
</template>
