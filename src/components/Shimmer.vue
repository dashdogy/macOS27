<script lang="ts" setup>
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/utils'

interface ShimmerProps {
  as?: keyof HTMLElementTagNameMap
  class?: HTMLAttributes['class']
  duration?: number
  spread?: number
  delay?: number
  repeatDelay?: number
  reverse?: boolean
}

const props = defineProps<ShimmerProps>()
const preference = usePreference()

const animDuration = computed(() => `${(props.duration ?? 1200) + (props.repeatDelay ?? 0)}ms`)
const animDelay = computed(() => `${props.delay ?? 200}ms`)
</script>

<template>
  <div
    :class="cn(
      'relative inline-block bg-[length:250%_100%,auto]',
      '[--base-color:#a1a1aa] [--base-gradient-color:#000]',
      '[--bg:linear-gradient(90deg,#0000_calc(50%-var(--spread)),var(--base-gradient-color),#0000_calc(50%+var(--spread)))] [background-repeat:no-repeat,padding-box]',
      'dark:[--base-color:#71717a] dark:[--base-gradient-color:#ffffff] dark:[--bg:linear-gradient(90deg,#0000_calc(50%-var(--spread)),var(--base-gradient-color),#0000_calc(50%+var(--spread)))]',
      preference.animationsEnabled ? (props.reverse ? 'shimmer-animate-reverse' : 'shimmer-animate') : '',
      props.class,
    )"
    :style="{
      '--spread': `${props.spread ?? 20}px`,
      '--shimmer-duration': animDuration,
      '--shimmer-delay': animDelay,
      'background-image': 'var(--bg), linear-gradient(var(--base-color), var(--base-color))',
    }"
  >
    <slot />
  </div>
</template>

<style scoped>
@keyframes shimmer-slide {
  from { background-position: 100% center; }
  to { background-position: 0% center; }
}
@keyframes shimmer-slide-reverse {
  from { background-position: 0% center; }
  to { background-position: 100% center; }
}
.shimmer-animate {
  will-change: background-position;
  animation: shimmer-slide var(--shimmer-duration, 1200ms) linear var(--shimmer-delay, 200ms) infinite;
}
.shimmer-animate-reverse {
  will-change: background-position;
  animation: shimmer-slide-reverse var(--shimmer-duration, 1200ms) linear var(--shimmer-delay, 200ms) infinite;
}
</style>
