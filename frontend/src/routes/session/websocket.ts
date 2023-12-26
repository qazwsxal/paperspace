import { count } from './stores';


export function storeSubscribe(socket: WebSocket) {
// Listen for messages
socket.addEventListener('message', function (event) {
    let message = JSON.parse(event.data)
    console.log(message)
    count.set(message);
})
}
