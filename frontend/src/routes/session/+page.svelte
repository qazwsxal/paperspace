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

AAAAAA Looking at {uuid}, current value is {$count}

<Incrementer {socket}></Incrementer>
<Decrementer {socket}></Decrementer>
<Resetter {socket}></Resetter>
