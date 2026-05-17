/**
 * WebRTC playback client for the "This Browser" output.
 *
 * The browser is a detachable output — when this client tears down, the
 * backend's other sessions are unaffected.
 *
 * Flow:
 *   1. createOffer() on a fresh RTCPeerConnection with a recvonly audio
 *      transceiver
 *   2. POST the SDP to /api/webrtc/offer
 *   3. setRemoteDescription with the backend's answer
 *   4. Pipe the inbound MediaStream into an <audio> element
 *
 * Closing the page or calling stop() will close the peer connection,
 * which the backend handles gracefully.
 */
export class BrowserPlayback {
  private pc: RTCPeerConnection | null = null;
  private audio: HTMLAudioElement | null = null;

  async start(audioEl: HTMLAudioElement): Promise<void> {
    this.audio = audioEl;

    const pc = new RTCPeerConnection({
      iceServers: [{ urls: 'stun:stun.l.google.com:19302' }]
    });

    pc.addTransceiver('audio', { direction: 'recvonly' });

    pc.ontrack = (e) => {
      if (audioEl.srcObject !== e.streams[0]) {
        audioEl.srcObject = e.streams[0];
        audioEl.play().catch((err) => console.warn('audio.play() rejected', err));
      }
    };

    const offer = await pc.createOffer();
    await pc.setLocalDescription(offer);
    // Wait for ICE gathering so we ship a complete SDP (no trickle).
    await new Promise<void>((resolve) => {
      if (pc.iceGatheringState === 'complete') return resolve();
      const onChange = () => {
        if (pc.iceGatheringState === 'complete') {
          pc.removeEventListener('icegatheringstatechange', onChange);
          resolve();
        }
      };
      pc.addEventListener('icegatheringstatechange', onChange);
    });

    const local = pc.localDescription;
    if (!local) throw new Error('local SDP missing');

    const res = await fetch('/api/webrtc/offer', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ sdp: local.sdp })
    });
    if (!res.ok) {
      const text = await res.text().catch(() => '');
      throw new Error(`backend rejected offer: ${res.status} ${text}`);
    }
    const { sdp } = (await res.json()) as { sdp: string };
    await pc.setRemoteDescription({ type: 'answer', sdp });

    this.pc = pc;
  }

  stop(): void {
    if (this.audio) {
      this.audio.srcObject = null;
    }
    if (this.pc) {
      this.pc.close();
      this.pc = null;
    }
  }
}
