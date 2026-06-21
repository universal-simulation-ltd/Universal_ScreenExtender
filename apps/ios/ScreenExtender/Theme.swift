import SwiftUI

/// Shared brand styling + per-mode display metadata, so the home screen, mode
/// picker and connected headers all describe a mode the same way.

extension Color {
    /// UNI·SIM brand orange (#e05504). The asset-catalog AccentColor drives the
    /// global tint; this literal is for the odd place that needs the colour direct.
    static let brandOrange = Color(red: 224 / 255, green: 85 / 255, blue: 4 / 255)
    /// The dark slate the app icon sits on — used as a logo backdrop tint.
    static let brandSlate = Color(red: 15 / 255, green: 23 / 255, blue: 42 / 255)
}

extension Mode {
    /// Short title shown in pickers, saved-host subtitles and connected headers.
    var label: String {
        switch self {
        case .clicker:      "Clicker"
        case .viewer:       "Mirror"
        case .control:      "Remote control"
        case .trackpad:     "Trackpad"
        case .secondScreen: "Second screen"
        }
    }

    /// One-line description of what the mode does (mode picker).
    var subtitle: String {
        switch self {
        case .clicker:      "Presentation remote — next/previous, blank, slide previews"
        case .viewer:       "Watch the host's screen (view only)"
        case .control:      "See the screen and control it (mouse + keys)"
        case .trackpad:     "Use the phone as a touchpad — move, tap, scroll"
        case .secondScreen: "Use the phone as an extra display (needs a virtual-display driver on the PC)"
        }
    }

    /// SF Symbol that represents the mode in lists and headers.
    var systemImage: String {
        switch self {
        case .clicker:      "rectangle.on.rectangle.angled"
        case .viewer:       "eye"
        case .control:      "cursorarrow.motionlines"
        case .trackpad:     "hand.draw"
        case .secondScreen: "display.2"
        }
    }
}

/// Resolve a stored raw mode string to its display label (saved hosts may have an
/// empty/unknown mode, in which case the raw text is shown as-is).
func modeLabel(_ raw: String) -> String {
    Mode(rawValue: raw)?.label ?? raw
}

/// The bar across the top of a connected (non-streaming) screen: a tappable mode
/// chip on the left to go back and re-pick, and a Disconnect button on the right.
/// Shared by the Clicker and Trackpad screens so both read identically.
struct ConnectedHeader: View {
    let mode: Mode
    var onSwitchMode: (() -> Void)? = nil
    let onDisconnect: () -> Void

    var body: some View {
        HStack(spacing: 12) {
            Button { onSwitchMode?() } label: {
                HStack(spacing: 6) {
                    Image(systemName: mode.systemImage)
                    Text(mode.label).fontWeight(.semibold)
                    if onSwitchMode != nil {
                        Image(systemName: "chevron.down").font(.caption2.weight(.bold))
                    }
                }
                .font(.subheadline)
                .padding(.horizontal, 12).padding(.vertical, 7)
                .background(Color.brandOrange.opacity(0.12), in: Capsule())
                .foregroundStyle(Color.brandOrange)
            }
            .buttonStyle(.plain)
            .disabled(onSwitchMode == nil)

            Spacer()

            Button(role: .destructive, action: onDisconnect) {
                Text("Disconnect").fontWeight(.medium)
            }
            .buttonStyle(.bordered)
            .tint(.red)
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 10)
    }
}
