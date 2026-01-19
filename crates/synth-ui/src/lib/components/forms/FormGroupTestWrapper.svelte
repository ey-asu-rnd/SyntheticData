<script lang="ts">
	import FormGroup from './FormGroup.svelte';

	let {
		label,
		htmlFor = '',
		helpText = '',
		error = '',
		required = false,
		childContent = 'Test Child Content',
	}: {
		label: string;
		htmlFor?: string;
		helpText?: string;
		error?: string;
		required?: boolean;
		childContent?: string;
	} = $props();
</script>

<FormGroup {label} {htmlFor} {helpText} {error} {required}>
	<input type="text" id={htmlFor} placeholder={childContent} data-testid="child-input" />
</FormGroup>
