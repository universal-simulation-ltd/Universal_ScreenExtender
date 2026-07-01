import AVFoundation
import SwiftUI

/// A live camera viewfinder that decodes QR codes and calls `onScan` with the
/// first result, then stops. Present as a full-screen sheet from the connect screen.
///
/// A big square viewfinder with the Universal logo in its centre frames the shot
/// and helps line the code up (the host's QR shows the same icon). The square and
/// its dimmed surround are purely visual — the camera still scans the whole frame,
/// so a code anywhere on screen is decoded.
struct QRScannerView: View {
    let onScan: (String) -> Void
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        GeometryReader { geo in
            let side = min(geo.size.width, geo.size.height) * 0.72
            ZStack {
                QRCaptureView(onScan: { text in
                    dismiss()
                    onScan(text)
                })
                .frame(maxWidth: .infinity, maxHeight: .infinity)

                // Dim everything outside the square (visual only — scanning still
                // uses the whole frame).
                Rectangle()
                    .fill(Color.black.opacity(0.55))
                    .reverseMask {
                        RoundedRectangle(cornerRadius: 28, style: .continuous)
                            .frame(width: side, height: side)
                    }

                // The square viewfinder with the Universal icon in its centre as an
                // alignment guide.
                RoundedRectangle(cornerRadius: 28, style: .continuous)
                    .stroke(Color.white.opacity(0.9), lineWidth: 3)
                    .frame(width: side, height: side)

                Image("AppLogo")
                    .resizable()
                    .scaledToFit()
                    .frame(width: side * 0.32, height: side * 0.32)
                    .opacity(0.85)

                Text("Scan a Universal QR code")
                    .font(.headline)
                    .foregroundStyle(.white)
                    .padding(.horizontal, 16)
                    .padding(.vertical, 8)
                    .background(.ultraThinMaterial, in: Capsule())
                    .offset(y: side / 2 + 44)

                VStack {
                    Spacer()
                    Button("Cancel") { dismiss() }
                        .padding()
                        .background(.ultraThinMaterial, in: Capsule())
                        .padding(.bottom, 40)
                }
            }
        }
        .ignoresSafeArea()
    }
}

private extension View {
    /// Punch the given shape out of this view, leaving a transparent hole.
    @ViewBuilder
    func reverseMask<Mask: View>(@ViewBuilder _ mask: () -> Mask) -> some View {
        self.mask {
            Rectangle()
                .overlay(alignment: .center) {
                    mask().blendMode(.destinationOut)
                }
                .compositingGroup()
        }
    }
}

// MARK: - AVFoundation bridge

private struct QRCaptureView: UIViewRepresentable {
    let onScan: (String) -> Void

    func makeUIView(context: Context) -> QRCaptureUIView {
        let view = QRCaptureUIView()
        view.onScan = onScan
        return view
    }

    func updateUIView(_ uiView: QRCaptureUIView, context: Context) {}
}

final class QRCaptureUIView: UIView {
    var onScan: ((String) -> Void)?

    private let captureSession = AVCaptureSession()
    private var previewLayer: AVCaptureVideoPreviewLayer?
    private var started = false

    override func didMoveToWindow() {
        super.didMoveToWindow()
        guard window != nil, !started else { return }
        started = true
        setup()
    }

    private func setup() {
        guard let device = AVCaptureDevice.default(for: .video),
              let input = try? AVCaptureDeviceInput(device: device),
              captureSession.canAddInput(input) else { return }
        captureSession.addInput(input)

        let output = AVCaptureMetadataOutput()
        guard captureSession.canAddOutput(output) else { return }
        captureSession.addOutput(output)
        output.setMetadataObjectsDelegate(self, queue: .main)
        output.metadataObjectTypes = [.qr]

        let preview = AVCaptureVideoPreviewLayer(session: captureSession)
        preview.videoGravity = .resizeAspectFill
        preview.frame = bounds
        layer.insertSublayer(preview, at: 0)
        previewLayer = preview

        DispatchQueue.global(qos: .userInitiated).async { self.captureSession.startRunning() }
    }

    override func layoutSubviews() {
        super.layoutSubviews()
        previewLayer?.frame = bounds
    }
}

extension QRCaptureUIView: AVCaptureMetadataOutputObjectsDelegate {
    func metadataOutput(
        _ output: AVCaptureMetadataOutput,
        didOutput objects: [AVMetadataObject],
        from connection: AVCaptureConnection
    ) {
        guard let obj = objects.first as? AVMetadataMachineReadableCodeObject,
              let text = obj.stringValue, !text.isEmpty else { return }
        captureSession.stopRunning()
        onScan?(text)
    }
}
