<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core"
import { listen } from "@tauri-apps/api/event"
import { useColorMode } from "@vueuse/core"
import { onMounted, onUnmounted, ref } from "vue"
import { toast } from "vue-sonner"
import { Button } from "@/components/ui/button"
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog"
import { Toaster } from "@/components/ui/sonner"
import { useUpdater } from "@/composables/useUpdater"
import UpdaterDialog from "@/features/updater/UpdaterDialog.vue"
import "vue-sonner/style.css"

interface HistoryEntry {
  id: number
  text: string
  timestamp: string
}

interface ShortcutConfig {
  modifiers: string[]
  key: string
}

interface AppConfig {
  shortcut: ShortcutConfig
  dark_mode: boolean
}

const history = ref<HistoryEntry[]>([])
const recording = ref(false)
const recordedKeys = ref<string[]>([])
const recordedCode = ref("")
const error = ref("")
const showAbout = ref(false)
const showClearConfirm = ref(false)
const currentShortcut = ref<ShortcutConfig>({ modifiers: [], key: "KeyA" })

const colorMode = useColorMode()
const updater = useUpdater()

listen("set-theme", (event) => {
  const payload = event.payload
  if (typeof payload === "boolean") {
    colorMode.value = payload ? "dark" : "light"
  }
  else if (payload === "light" || payload === "dark" || payload === "auto") {
    colorMode.value = payload
  }
})

listen("record-shortcut", () => {
  startRecording()
})

listen("confirm-clear", () => {
  confirmClear()
})

listen("show-about", () => {
  showAbout.value = true
})

listen("check-update", () => {
  updater.checkForUpdates()
})

function modToSymbol(mod: string): string {
  switch (mod) {
    case "Alt": return "⌥"
    case "Control": return "⌃"
    case "Shift": return "⇧"
    case "Super": return "⌘"
    default: return mod
  }
}

function formatShortcut(cfg: ShortcutConfig): string {
  return [...cfg.modifiers.map(modToSymbol), cfg.key.replace("Key", "")].join("")
}

function startRecording() {
  recording.value = true
  showAbout.value = false
  showClearConfirm.value = false
  recordedKeys.value = []
  recordedCode.value = ""
  error.value = ""
}

function cancelRecording() {
  recording.value = false
}

function onKeydown(e: KeyboardEvent) {
  if (!recording.value)
    return
  e.preventDefault()
  e.stopPropagation()

  if (e.key === "Escape") {
    cancelRecording()
    return
  }

  const mods: string[] = []
  if (e.altKey)
    mods.push("Alt")
  if (e.ctrlKey)
    mods.push("Control")
  if (e.shiftKey)
    mods.push("Shift")
  if (e.metaKey)
    mods.push("Super")

  const code = e.code

  if (mods.length === 0 || !code.startsWith("Key")) {
    error.value = "Include a modifier (⌘⌥⌃⇧) + a letter"
    return
  }

  error.value = ""
  recordedKeys.value = mods
  recordedCode.value = code
}

async function confirmShortcut() {
  if (recordedKeys.value.length === 0 || !recordedCode.value)
    return

  const sameMods
    = recordedKeys.value.length === currentShortcut.value.modifiers.length
      && recordedKeys.value.every(m => currentShortcut.value.modifiers.includes(m))
  const sameKey = recordedCode.value === currentShortcut.value.key

  if (sameMods && sameKey) {
    error.value = "Same as current shortcut"
    return
  }

  try {
    currentShortcut.value = await invoke<ShortcutConfig>("set_shortcut", {
      modifiers: recordedKeys.value,
      key: recordedCode.value,
    })
    recording.value = false
    toast.success("Shortcut registered successfully")
  }
  catch (e: any) {
    error.value = String(e)
    toast.error("Failed to register shortcut")
  }
}

async function loadHistory() {
  try {
    history.value = await invoke<HistoryEntry[]>("get_history")
  }
  catch (e) {
    console.error("Failed to load history", e)
  }
}

function confirmClear() {
  showClearConfirm.value = true
}

async function clearHistory() {
  showClearConfirm.value = false
  try {
    await invoke("clear_history")
    history.value = []
    toast.success("OCR history cleared")
  }
  catch (e) {
    console.error("Failed to clear history", e)
    toast.error("Failed to clear history")
  }
}

async function copyText(text: string) {
  try {
    const { writeText } = await import("@tauri-apps/plugin-clipboard-manager")
    await writeText(text)
  }
  catch {
    try {
      await navigator.clipboard.writeText(text)
    }
    catch {
      console.error("Failed to copy")
      toast.error("Failed to copy text")
    }
  }
}

function formatPreview(text: string): string {
  return text.replace(/\n+/g, " ").trim()
}

async function loadShortcut() {
  try {
    currentShortcut.value = await invoke<ShortcutConfig>("get_shortcut")
  }
  catch {
    // use default
  }
}

async function loadTheme() {
  try {
    const cfg = await invoke<AppConfig>("get_config")
    colorMode.value = cfg.dark_mode ? "dark" : "light"
  }
  catch {
    // fallback
  }
}

let pollTimer: ReturnType<typeof setInterval> | undefined

onMounted(() => {
  loadTheme()
  loadHistory()
  loadShortcut()
  pollTimer = setInterval(loadHistory, 2000)
  document.addEventListener("keydown", onKeydown)
})

onUnmounted(() => {
  if (pollTimer)
    clearInterval(pollTimer)
  document.removeEventListener("keydown", onKeydown)
})
</script>

<template>
  <div class="w-full h-screen bg-white dark:bg-gray-900 text-gray-800 dark:text-gray-200 overflow-hidden flex flex-col select-none">
    <div class="h-7 shrink-0" />

    <div class="flex-1 overflow-y-auto px-3 pb-3 space-y-1.5 scrollbar-thin">
      <!-- Shortcut recorder -->
      <div v-if="recording" class="flex flex-col items-center justify-center h-48 text-center">
        <p class="text-[13px] font-medium text-gray-600 dark:text-gray-300 mb-3">
          Press new shortcut
        </p>
        <div v-if="recordedKeys.length > 0 && recordedCode" class="flex items-center gap-1 mb-3">
          <span
            v-for="mod in recordedKeys"
            :key="mod"
            class="px-2 py-1 text-[13px] font-medium rounded bg-gray-100 dark:bg-gray-700 border border-gray-200 dark:border-gray-600"
          >
            {{ modToSymbol(mod) }}
          </span>
          <span class="px-2 py-1 text-[13px] font-medium rounded bg-blue-50 dark:bg-blue-900/30 border border-blue-200 dark:border-blue-700 text-blue-600 dark:text-blue-400">
            {{ recordedCode.replace("Key", "") }}
          </span>
        </div>
        <p v-else class="text-[12px] text-gray-400 mb-3">
          e.g. ⌘⇧X
        </p>
        <p v-if="error" class="text-[11px] text-red-400 mb-2">
          {{ error }}
        </p>
        <div v-if="recordedKeys.length > 0 && recordedCode" class="flex items-center gap-2">
          <Button
            size="sm"
            @click="confirmShortcut"
          >
            Confirm
          </Button>
          <Button
            size="sm"
            variant="ghost"
            @click="cancelRecording"
          >
            Cancel
          </Button>
        </div>
        <p v-else class="text-[11px] text-gray-400">
          Press Esc to cancel
        </p>
      </div>

      <!-- About dialog -->
      <div v-else-if="showAbout" class="flex flex-col items-center justify-center h-48 text-center px-6">
        <p class="text-[14px] font-semibold mb-1">
          TrayOCR
        </p>
        <p class="text-[11px] text-gray-400 mb-3">
          v0.1.0
        </p>
        <p class="text-[11px] text-gray-500 leading-relaxed">
          Minimal OCR screenshot tool.<br>
          Powered by system VisionKit.
        </p>
        <Button
          class="mt-4"
          variant="secondary"
          size="sm"
          @click="showAbout = false"
        >
          Close
        </Button>
      </div>

      <!-- History view -->
      <template v-if="!recording && !showAbout">
        <div v-if="history.length === 0" class="flex flex-col items-center justify-center h-48 text-gray-300 dark:text-gray-600 px-6">
          <svg class="w-10 h-10 mb-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
            <polyline points="14 2 14 8 20 8" />
            <line x1="12" y1="18" x2="12" y2="12" />
            <line x1="9" y1="15" x2="15" y2="15" />
          </svg>
          <p class="text-[13px] font-medium text-gray-400 dark:text-gray-500 mb-1">
            No OCR history yet
          </p>
          <p class="text-[11px] text-gray-300 dark:text-gray-600 text-center leading-relaxed mb-3">
            Select an area with<br>
            <span class="inline-flex items-center gap-0.5 px-1.5 py-0.5 rounded bg-gray-100 dark:bg-gray-700 font-mono text-[11px] text-gray-500 dark:text-gray-400">
              {{ formatShortcut(currentShortcut) }}
            </span>
          </p>
          <Button
            variant="outline"
            size="sm"
            class="mt-2 rounded-full text-[11px] h-7 px-3 text-muted-foreground"
            @click="updater.checkForUpdates"
          >
            Check for updates
          </Button>
        </div>

        <div
          v-for="item in history" :key="item.id"
          class="group relative bg-white dark:bg-gray-800/50 rounded-xl border border-gray-100 dark:border-gray-700/50 p-3 cursor-pointer transition-all hover:border-blue-200 dark:hover:border-blue-500/50 hover:shadow-sm active:scale-[0.98]"
          @click="copyText(item.text)"
        >
          <p class="text-[12px] leading-relaxed text-gray-700 dark:text-gray-300 line-clamp-4 break-words">
            {{ formatPreview(item.text) }}
          </p>
          <div class="flex items-center justify-between mt-1.5">
            <span class="text-[10px] text-gray-400 dark:text-gray-500">{{ item.timestamp }}</span>
            <span class="text-[10px] text-blue-400 dark:text-blue-400 opacity-0 group-hover:opacity-100 transition-opacity">
              Click to copy
            </span>
          </div>
        </div>
      </template>
    </div>

    <div v-if="!recording && !showAbout && history.length > 0" class="px-3 py-2 border-t border-gray-100 dark:border-gray-800 flex justify-center">
      <Button
        variant="ghost"
        size="sm"
        class="w-full text-xs text-muted-foreground hover:text-destructive dark:hover:text-destructive"
        @click="confirmClear"
      >
        Clear history
      </Button>
    </div>
  </div>

  <Dialog v-model:open="showClearConfirm">
    <DialogContent class="max-w-[300px] rounded-lg">
      <DialogHeader>
        <DialogTitle class="text-[14px]">
          Clear History
        </DialogTitle>
        <DialogDescription class="text-[12px]">
          Are you sure you want to clear all OCR history?
        </DialogDescription>
      </DialogHeader>
      <DialogFooter class="flex flex-row justify-end gap-2 pt-2">
        <Button
          variant="ghost"
          size="sm"
          @click="showClearConfirm = false"
        >
          Cancel
        </Button>
        <Button
          variant="destructive"
          size="sm"
          @click="clearHistory"
        >
          Clear
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>

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

<style>
.scrollbar-thin::-webkit-scrollbar {
  width: 3px;
}
.scrollbar-thin::-webkit-scrollbar-track {
  background: transparent;
}
.scrollbar-thin::-webkit-scrollbar-thumb {
  background: #e5e7eb;
  border-radius: 99px;
}
.dark .scrollbar-thin::-webkit-scrollbar-thumb {
  background: #4b5563;
}
.line-clamp-4 {
  display: -webkit-box;
  -webkit-line-clamp: 4;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
</style>
