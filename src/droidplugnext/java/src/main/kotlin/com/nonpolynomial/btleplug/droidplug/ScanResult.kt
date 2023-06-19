package com.nonpolynomial.btleplug.droidplug

class ScanResult(
        val address: String,
        val addressType: Int?,
        val localName: String?,
        var txPowerLevel: Int?,
        val rssi: Int?,
        val manufacturerData: HashMap<Int, ByteArray>,
        val serviceData: HashMap<String, ByteArray>,
        val services: List<String>,
) {
}
