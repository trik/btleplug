package com.nonpolynomial.btleplug.droidplug

class Characteristic(
        val uuid: String,
        val properties: Int,
        val descriptors: List<String>,
) {
}