The buddy board features four thermistors measuring the bed, board, hotend and pinda temperatures. The pinda thermistor has been deprecated since the addition of the SuperPINDA on the Prusa printers.

The thermistors are configured in a pull-up resistor arrangement. The relationship between voltage and resistance is:

\\[
  V_{\text{sample}} = V_{\text{in}} \left(\frac{R_{\text{th}}}{R_{\text{th}}+R_{\text{pu}}}\right)
  \tag{1}
\\]

where \\(V_{\text{sample}}\\) is the voltage to be sampled, \\(V_{\text{in}}\\) is the voltage powering the circuit, \\(R_{\text{th}}\\) is the resistance of the thermistor and \\(R_{\text{pu}}\\) is the resistance of the pull-up resistor.

\\(V_{\text{sample}}\\) is an analog signal that needs to be converted into a digital signal using one of the STM32's Analog-to-Digital Converters (ADCs). The ADC provides an integer range of values across the 0V-\\(V_{\text{in}}\\) continuous range. Therefore, we can modify Equation 1 by substituting \\(V_{\text{in}}\\) and \\(V_{\text{sample}}\\) with the max integer that the ADC can record and the sample ADC reading.

\\[
 ADC_{\text{sample}} = ADC_{\text{max}} \left(\frac{R_{\text{th}}}{R_{\text{th}}+R_{\text{pu}}}\right)
\tag{2}
\\]

Now we need to re-arrange Equation 2 so we can determine the resistance of the thermistor \\(R_{\text{th}}\\).

\\[
ADC_{\text{sample}}\left(R_{\text{th}}+R_{\text{pu}}\right) = ADC_{\text{max}} R_{\text{th}}
\\]

\\[
ADC_{\text{sample}}R_{\text{th}} + ADC_{\text{sample}}R_{\text{pu}} = ADC_{\text{max}} R_{\text{th}}
\\]


\\[
ADC_{\text{sample}}R_{\text{pu}} = ADC_{\text{max}} R_{\text{th}} - ADC_{\text{sample}}R_{\text{th}}
\\]

\\[
ADC_{\text{sample}}R_{\text{pu}} = \left(ADC_{\text{max}} - ADC_{\text{sample}}\right)R_{\text{th}}
\\]

\\[
\frac{ADC_{\text{sample}}R_{\text{pu}}}{\left(ADC_{\text{max}} - ADC_{\text{sample}}\right)} = R_{\text{th}}
\tag{3}
\\]

We can then use \\(R_{\text{th}}\\) to determine the temperature of the thermistor by using [Steinhart-Hart](https://en.wikipedia.org/wiki/Steinhart%E2%80%93Hart_equation) equation.

\\[
R = R_0e^{\beta\left(\frac{1}{T}-\frac{1}{T_0}\right)}
\tag{4}
\\]

Where \\(R\\) is the resistance we calculated, \\(R_0\\) is a reference resistance, \\(\beta\\) is the Steinhart-Hart coefficient for the resistor, \\(T\\) is the temperature of the resistor, and \\(T_0\\) is a reference temperture. Reference values are typically calculated at 25C.

Re-arranging for \\(T\\) gives:

\\[
\frac{R}{R_0} = e^{\beta\left(\frac{1}{T}-\frac{1}{T_0}\right)}
\\]

\\[
ln\left(\frac{R}{R_0}\right) = \beta\left(\frac{1}{T}-\frac{1}{T_0}\right)
\\]

\\[
\frac{ln\left(\frac{R}{R_0}\right)}{\beta} = \frac{1}{T}-\frac{1}{T_0}
\\]

\\[
\frac{ln\left(\frac{R}{R_0}\right)}{\beta} + \frac{1}{T_0} = \frac{1}{T}
\\]

\\[
\frac{1}{\frac{1}{\beta}ln\left(\frac{R}{R_0}\right) + \frac{1}{T_0}} = T
\tag{5}
\\]

| Thermistor | Type | R | \\(\beta\\) |
| --- | --- | --- | --- |
| Bed | [Semitec 104NT-4-R025H42G](https://atcsemitec.co.uk/wp-content/uploads/2019/01/Semitec-NT-4-Glass-NTC-Thermistor.pdf) | 100kΩ | 4267K |
| Board | | | |
| Hotend | [Semitec 104GT-2-40231](https://download.lulzbot.com/retail_parts/Completed_Parts/100k_Semitec_NTC_Thermistor_235mm_KT-CP0110/GT-2-glass-thermistors.pdf) | 100kΩ  | 4267K |


## References

- [Embedded Rustacean](https://blog.theembeddedrustacean.com/embedded-rust-embassy-analog-sensing-with-adcs)
