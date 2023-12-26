<script lang="ts">
    export let socket: WebSocket;
    
	function decrement() {
        if (socket) {
		socket.send("{\"Decrement\":[]}");
        }
	}


</script>

<button on:click={decrement}>
	-
</button>
