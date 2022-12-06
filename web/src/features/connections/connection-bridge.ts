export class WSBridge {
	intiface_ws: WebSocket;
	backend_ws: WebSocket;

	constructor(intiface_url: string, backend_url: string) {
		this.intiface_ws = new WebSocket(intiface_url);
		this.backend_ws = new WebSocket(backend_url);

		this.intiface_ws.onmessage = (event) => {
			this.backend_ws.send(event.data);
		};

		this.backend_ws.onmessage = (event) => {
			this.intiface_ws.send(event.data);
		};
	}
}