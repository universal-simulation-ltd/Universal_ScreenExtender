package com.universalsim.extender

import android.view.View
import com.journeyapps.barcodescanner.CaptureActivity
import com.journeyapps.barcodescanner.DecoratedBarcodeView

/**
 * zxing-android-embedded's built-in capture screen is locked to landscape (its
 * manifest entry forces it), so even `setOrientationLocked(false)` opens sideways.
 * This subclass is declared `android:screenOrientation="portrait"` in our manifest
 * and pointed at via `ScanOptions.setCaptureActivity(...)`, so the scanner opens
 * upright.
 *
 * It also swaps in our own layout (`portrait_capture.xml`): a big square with the
 * Universal logo in its centre as an alignment guide, and a "Scan a Universal QR
 * code" caption. That overlay is purely visual — we set the decode margin to zero
 * so the whole camera frame is scanned, and hide zxing's default viewfinder and
 * status text so only our square shows.
 */
class PortraitCaptureActivity : CaptureActivity() {
    override fun initializeContent(): DecoratedBarcodeView {
        setContentView(R.layout.portrait_capture)
        val scannerView = findViewById<DecoratedBarcodeView>(R.id.zxing_barcode_scanner)
        // Scan the entire frame, not a centred crop — our square is just a guide.
        scannerView.barcodeView.setMarginFraction(0.0)
        // Hide the library's own viewfinder (mask + laser) and status text; our
        // overlay replaces them.
        scannerView.viewFinder.visibility = View.GONE
        scannerView.statusView.visibility = View.GONE
        return scannerView
    }
}
