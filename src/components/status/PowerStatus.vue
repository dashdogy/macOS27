<script setup lang="ts">
import { Card } from '@/components/ui/card'
import { formatChargingDuration } from '@/lib/format'
import { addSeconds, format } from 'date-fns'

const isPopover = inject('isPopover', false)
const div = h('div')
const power = usePower()

const instantDuration = computed(() => {
  const secs = power.value.instantTimeRemain?.secs ?? 0
  if (secs <= 0 || secs > 86400) return null
  return formatChargingDuration(secs)
})

const osDuration = computed(() => {
  const secs = power.value.timeRemain?.secs ?? 0
  if (secs <= 0 || secs > 86400) return null
  return formatChargingDuration(secs)
})

const untilTime = computed(() => {
  const secs = power.value.timeRemain?.secs ?? 0
  if (secs <= 0 || secs > 86400) return null
  return format(addSeconds(new Date(), secs), 'HH:mm')
})

const wattageTooltip = computed(() => {
  const w = (power.value.isCharging ? power.value.systemIn : power.value.systemLoad).toFixed(1)
  return instantDuration.value ? `${w}w (${instantDuration.value} at current rate)` : `${w}w`
})
</script>

<template>
  <Component
    :is="isPopover ? div : Card"
    class="min-w-80"
    :class="{ 'flex-1 bg-transparent border-none shadow-none': isPopover }"
  >
    <template v-if="isPopover">
      <!-- Compact popover layout -->
      <div v-if="power.isLoading" class="p-3 space-y-3">
        <Skeleton class="w-full h-6" />
        <Skeleton class="w-full h-3" />
        <Skeleton class="w-full h-4" />
      </div>
      <div v-else class="p-4 space-y-3 font-sans">
        <!-- Row 1: Battery % + Wattage -->
        <div class="flex items-center justify-between text-base">
          <div class="flex items-center gap-2">
            <span class="font-semibold">🔋 {{ power.batteryLevel.toFixed(0) }}%</span>
            <span class="text-foreground/50">
              {{ power.isCharging ? power.adapterName : $t('status.on_battery') }}
            </span>
          </div>
          <CommonTooltip :content="wattageTooltip" as-child>
            <span class="font-semibold tabular-nums cursor-default">
              {{ (power.isCharging ? power.systemIn : power.systemLoad).toFixed(1) }}w
            </span>
          </CommonTooltip>
        </div>

        <!-- Row 2: Power bar -->
        <PowerStatusBar />

        <!-- Row 3: Time estimates -->
        <div class="flex items-center justify-between text-sm text-foreground/60">
          <span v-if="osDuration">
            est. <span class="font-semibold" :class="power.isCharging ? 'text-blue-500' : 'text-foreground'">{{ osDuration }}</span>
            {{ power.isCharging ? $t('status.to_full') : $t('status.to_empty') }}
          </span>
          <span v-else>—</span>
          <span v-if="untilTime">until <span class="font-semibold text-foreground">{{ untilTime }}</span></span>
        </div>

        <!-- Row 4: Power breakdown (local only) -->
        <div v-if="!power.isRemote" class="flex items-center gap-3 text-sm text-foreground/50">
          <span>Screen <span class="font-semibold text-foreground/80">{{ (power.brightnessPower || 0).toFixed(1) }}w</span></span>
          <span>·</span>
          <span>SoC <span class="font-semibold text-foreground/80">{{ (power.heatpipePower || 0).toFixed(1) }}w</span></span>
          <span v-if="power.isCharging">·</span>
          <span v-if="power.isCharging">Loss <span class="font-semibold text-foreground/80">{{ (power.efficiencyLoss || 0).toFixed(0) }}mw</span></span>
        </div>
      </div>
    </template>

    <template v-else>
      <!-- Full window layout (unchanged) -->
      <CardHeader class="space-y-0 pb-2 gap-y-0">
        <CardTitle class="flex items-center justify-between gap-2 text-base truncate">
          <PowerStatusTitle />
        </CardTitle>
        <CardDescription class="text-[10px] font-mono flex gap-[2px] items-center">
          <PowerStatusDescription />
        </CardDescription>
      </CardHeader>
      <CardContent class="space-y-3 mt-1">
        <PowerStatusNumber />
        <PowerStatusFooter />
      </CardContent>
    </template>
  </Component>
</template>
