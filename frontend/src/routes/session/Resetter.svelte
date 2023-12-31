<script lang="ts">
    export let socket: WebSocket;
    
	function reset() {
        if (socket) {
		socket.send("{\"Reset\":[]}");
        }
	}


</script>

<button class="btn join-item" on:click={reset}>
	0
</button>
