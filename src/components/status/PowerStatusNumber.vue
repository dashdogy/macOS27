<script setup lang="ts">
const power = usePower()
const { isLoading } = usePreferenceAsync()

const displayValue = computed(() =>
  (power.value.isCharging ? power.value.systemIn : power.value.systemLoad).toFixed(1)
)
</script>

<template>
  <div>
    <Skeleton v-if="power.isLoading || isLoading" class="w-40 h-[50px] mt-2" />
    <div v-else class="text-4xl leading-[54px] font-bold tabular-nums">
      {{ displayValue }}w
    </div>
  </div>
</template>
