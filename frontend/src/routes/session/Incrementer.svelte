<script lang="ts">
    export let socket: WebSocket;
    
	function increment() {
        if (socket) {
		socket.send("{\"Increment\":[]}");
        }
	}


</script>

<button class="btn join-item" on:click={increment}>
	+
</button>
