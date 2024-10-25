<script setup>
import { computed, getCurrentInstance } from 'vue';

const props = defineProps({
    d: {
        type: Date,
        required: true
    }
});

const formattedDateTime = computed(() => {
    return getCurrentInstance().appContext.config.globalProperties.$dateTimeFormatter.format(props.d);
});
</script>


<template>
{{ formattedDateTime }}
</template>