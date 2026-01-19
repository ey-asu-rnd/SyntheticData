<script lang="ts">
	import FormSection from './FormSection.svelte';

	let {
		title,
		description = '',
		collapsible = false,
		collapsed = false,
		childContent = 'Section Content',
	}: {
		title: string;
		description?: string;
		collapsible?: boolean;
		collapsed?: boolean;
		childContent?: string;
	} = $props();
</script>

<FormSection {title} {description} {collapsible} {collapsed}>
	<p data-testid="section-content">{childContent}</p>
</FormSection>
