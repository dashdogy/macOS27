<script setup lang="ts">
import type { UnwrapRef } from 'vue'
import { useI18n } from 'vue-i18n'

const power = usePower()
const { t } = useI18n()

const colorMap: Record<string, string> = {
  screen: 'bg-sky-500',
  heatpipe: 'bg-indigo-500',
  systemOther: 'bg-blue-700',
  systemTotal: 'bg-blue-600',
  batteryIn: 'bg-emerald-500',
  powerLoss: 'bg-orange-400',
}

// Fixed display order — never re-sorted
const displayOrder = ['screen', 'heatpipe', 'systemOther', 'systemTotal', 'batteryIn', 'powerLoss']

const localeMap = computed(() => ({
  screen: t('status.screen_power'),
  heatpipe: t('status.heatpipe_power'),
  systemOther: t('status.system_other'),
  batteryIn: t('status.battery_in'),
  systemTotal: t('status.system_total'),
  powerLoss: t('status.power_loss'),
}))

interface BarData {
  parts: [string, {
    value: number
    left: number
    color: string
    locale: string
  }][]
  sum: number
}

const data = ref<BarData>()
const handle = watchEffect(() => {
  const parts: Record<string, number> = {
    systemTotal: power.value.systemLoad,
  }

  if (power.value.isCharging) {
    parts.batteryIn = power.value.systemIn - parts.systemTotal
    parts.powerLoss = power.value.efficiencyLoss
  }

  if (!power.value.isRemote) {
    parts.screen = power.value.brightnessPower
    parts.heatpipe = power.value.heatpipePower
    parts.systemOther = parts.systemTotal - parts.screen - parts.heatpipe
    delete parts.systemTotal
  }

  const sum = power.value.isCharging
    ? power.value.systemIn + power.value.efficiencyLoss
    : power.value.systemLoad

  // Fixed order layout — accumulate left positions in display order
  let current = 0
  const ordered = displayOrder
    .filter(key => key in parts)
    .map(key => {
      const value = parts[key]
      const left = current / sum
      current += value
      return [key, {
        value,
        left,
        color: colorMap[key],
        locale: localeMap.value[key as keyof UnwrapRef<typeof localeMap>],
      }] as BarData['parts'][0]
    })

  data.value = { parts: ordered, sum }
})

const hovered = ref<string | null>(null)

watchEffect(() => {
  hovered.value ? handle.pause() : handle.resume()
})
</script>

<template>
  <!-- Charging: total = powerloss + screen + heatpipe + system other + battery in -->
  <!-- Not Charging: total = screen + heatpipe + system other -->
  <div
    v-if="data"
    class="relative h-3 overflow-hidden rounded flex transition-all duration-500"
    :class="[hovered ? '' : 'bg-blue-500']"
    @mouseleave="hovered = null"
  >
    <CommonTooltip
      v-for="[key, { value, left, color, locale }] in data.parts"
      :key="key"
      as-child
    >
      <template #popper>
        <span>{{ locale }}</span>
        <span class="ml-1 font-mono font-extrabold">{{ value.toFixed(1) }}w</span>
      </template>
      <div
        class="absolute top-0 bottom-0 transition-all duration-500"
        :class="[hovered && hovered !== key ? 'opacity-20' : 'opacity-100', color]"
        :style="{ width: `calc(${(value / data.sum) * 100}% + 1px)`, left: `${left * 100}%` }"
        @mouseover="hovered = key"
      />
    </CommonTooltip>
  </div>
</template>
