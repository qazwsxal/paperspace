<script lang="ts">
    export let socket: WebSocket;
    
	function increment() {
        if (socket) {
		socket.send("{\"Increment\":[]}");
        }
	}


</script>

<button on:click={increment}>
	+
</button>
