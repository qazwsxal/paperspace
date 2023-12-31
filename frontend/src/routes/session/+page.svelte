<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { count } from './stores';
	import { storeSubscribe } from './websocket';
	import Incrementer from './Incrementer.svelte';
	import Decrementer from './Decrementer.svelte';
	import Resetter from './Resetter.svelte';
	var uuid: string | null;
	var socket: WebSocket;
	onMount(() => {
		uuid = $page.url.searchParams.get('uuid');
		socket = new WebSocket(
			(window.location.protocol === 'https:' ? 'wss://' : 'ws://') +
				window.location.host +
				'/api/ws?uuid=' +
				uuid
		);
		// Connection opened
		socket.addEventListener('open', function (event) {
			console.log("It's open");
		});
		storeSubscribe(socket);
	});
</script>

<div class="flex flex-col w-full h-full lg:flex-row">
	<div class="grid flex-grow h-full place-items-center">
		<div role="tablist" class="tabs tabs-lifted w-full">
			<input type="radio" name="my_tabs_2" role="tab" class="tab" aria-label="Tab 1" />
			<div role="tabpanel" class="tab-content border-base-300 p-6">
				AAAAAA Looking at {uuid}, current value is {$count}
			</div>

			<input type="radio" name="my_tabs_2" role="tab" class="tab" aria-label="Tab 2" checked />
			<div role="tabpanel" class="tab-content border-base-300 p-6">
				AAAAAA Looking at {uuid}, current value is {$count}
			</div>

			<input type="radio" name="my_tabs_2" role="tab" class="tab" aria-label="Tab 3" />
			<div role="tabpanel" class="tab-content border-base-300 p-6">
				AAAAAA Looking at {uuid}, current value is {$count}
			</div>
		</div>
	</div>
	<div class="divider lg:divider-horizontal"></div>
	<div class="grid flex-grow h-full place-items-center">
		<div class="join">
			<Incrementer {socket}></Incrementer>
			<Decrementer {socket}></Decrementer>
			<Resetter {socket}></Resetter>
		</div>
	</div>
	<div class="divider lg:divider-horizontal"></div>
	<div class="grid flex-grow h-full place-items-center">
		AAAAAA Looking at {uuid}, current value is {$count}
	</div>
</div>
