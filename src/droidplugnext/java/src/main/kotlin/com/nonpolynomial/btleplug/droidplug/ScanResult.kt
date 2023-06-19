package com.nonpolynomial.btleplug.droidplug

class ScanResult {
    var address = String()
    var addressType: Int? = null
    var localName: String? = null
    var txPowerLevel: Int? = null
    var rssi: Int? = null
    val manufacturerData = HashMap<Int, ByteArray>()
    val serviceData = HashMap<String, ByteArray>()
    val services = ArrayList<String>()
}
