<script setup>
import { computed, getCurrentInstance } from 'vue';

const props = defineProps({
    n: {
        type: Number,
        required: true
    }
});

const formattedNumber = computed(() => {
    return getCurrentInstance().appContext.config.globalProperties.$numberFormatter.format(props.n);
});
</script>

<template>
{{ formattedNumber }}
</template>