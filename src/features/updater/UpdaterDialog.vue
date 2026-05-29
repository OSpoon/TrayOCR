<script setup lang="ts">
import { Button } from "@/components/ui/button"
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog"
import { Progress } from "@/components/ui/progress"

defineProps<{
  open: boolean
  title: string
  description: string
  progress: number | null
  busy: boolean
  pendingRestart: boolean
}>()

const emit = defineEmits<{
  "update:open": [value: boolean]
  "restart": []
}>()
</script>

<template>
  <Dialog :open="open" @update:open="emit('update:open', $event)">
    <DialogContent class="sm:max-w-md">
      <DialogHeader>
        <DialogTitle>{{ title }}</DialogTitle>
        <DialogDescription>
          {{ description }}
        </DialogDescription>
      </DialogHeader>
      <div v-if="progress !== null" class="pt-2">
        <Progress :model-value="progress" />
      </div>
      <DialogFooter class="pt-2">
        <div class="flex w-full items-center justify-end gap-2">
          <Button
            v-if="!pendingRestart"
            type="button"
            variant="secondary"
            :disabled="busy"
            @click="emit('update:open', false)"
          >
            OK
          </Button>
          <Button
            v-if="pendingRestart"
            type="button"
            :disabled="busy"
            @click="emit('restart')"
          >
            Restart now
          </Button>
        </div>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
