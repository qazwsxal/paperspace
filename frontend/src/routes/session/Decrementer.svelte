<script lang="ts">
    export let socket: WebSocket;
    
	function decrement() {
        if (socket) {
		socket.send("{\"Decrement\":[]}");
        }
	}


</script>

<button class="btn join-item" on:click={decrement}>
	-
</button>
