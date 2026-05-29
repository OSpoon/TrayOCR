import { getName, getVersion } from "@tauri-apps/api/app"
import { listen } from "@tauri-apps/api/event"
import { relaunch } from "@tauri-apps/plugin-process"
import { check } from "@tauri-apps/plugin-updater"
import { onBeforeUnmount, onMounted, ref } from "vue"
import { toast } from "vue-sonner"

const UPDATE_PENDING_RESTART_KEY = "app.updatePendingRestart"

export function useUpdater() {
  const dialogOpen = ref(false)
  const dialogTitle = ref("Software Update")
  const dialogDescription = ref("")
  const progress = ref<number | null>(null)
  const busy = ref(false)
  const pendingRestart = ref(false)

  const appName = ref("")
  const appVersion = ref("")

  async function openPendingRestartDialog() {
    dialogOpen.value = true
    dialogTitle.value = "Update installed"
    dialogDescription.value = "Please restart the app to complete the update."
    progress.value = 100
  }

  async function checkForUpdates() {
    if (busy.value)
      return

    if (pendingRestart.value) {
      await openPendingRestartDialog()
      return
    }

    busy.value = true
    dialogOpen.value = true
    dialogTitle.value = "Software Update"
    dialogDescription.value = "Checking for updates…"
    progress.value = null

    try {
      const update = await check()
      if (!update) {
        dialogTitle.value = "You're up to date!"
        dialogDescription.value = `${appName.value || "This app"} ${appVersion.value || ""} is the latest version.`.trim()
        return
      }

      dialogTitle.value = "Software Update"
      dialogDescription.value = `Found version ${update.version}. Downloading and installing…`
      progress.value = 0

      await update.downloadAndInstall((event) => {
        if (event.event === "Started") {
          progress.value = 0
        }
        else if (event.event === "Progress") {
          const data = (event as any).data as any
          const contentLength = Number(data?.contentLength ?? 0)
          const chunkLength = Number(data?.chunkLength ?? 0)
          const downloaded = Number(data?.downloaded ?? 0)

          if (contentLength > 0) {
            const value = Math.min(100, Math.max(0, Math.round(((downloaded || 0) / contentLength) * 100)))
            progress.value = value
          }
          else if (chunkLength > 0 && progress.value != null) {
            progress.value = Math.min(99, progress.value + 1)
          }
        }
        else if (event.event === "Finished") {
          progress.value = 100
        }
      })

      dialogTitle.value = "Update installed"
      dialogDescription.value = "Please restart the app to complete the update."
      progress.value = 100

      pendingRestart.value = true
      localStorage.setItem(UPDATE_PENDING_RESTART_KEY, "1")
    }
    catch (error) {
      const message = error instanceof Error ? error.message : String(error)
      dialogTitle.value = "Update failed"
      dialogDescription.value = message
      progress.value = null
    }
    finally {
      busy.value = false
    }
  }

  async function restartApp() {
    try {
      await relaunch()
    }
    catch (error) {
      const message = error instanceof Error ? error.message : String(error)
      toast.error("Restart failed", { description: message })
    }
  }

  let unlistenCheckUpdates: null | (() => void) = null

  onMounted(async () => {
    appName.value = await getName().catch(() => "")
    appVersion.value = await getVersion().catch(() => "")
    pendingRestart.value = localStorage.getItem(UPDATE_PENDING_RESTART_KEY) === "1"

    unlistenCheckUpdates = await listen("app://check-updates", () => {
      void checkForUpdates()
    })
  })

  onBeforeUnmount(() => {
    unlistenCheckUpdates?.()
    unlistenCheckUpdates = null
  })

  return {
    dialogOpen,
    dialogTitle,
    dialogDescription,
    progress,
    busy,
    pendingRestart,
    checkForUpdates,
    restartApp,
  }
}
