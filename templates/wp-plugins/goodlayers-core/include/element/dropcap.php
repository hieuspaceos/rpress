<?php
	/*	
	*	Goodlayers Item
	*/
	
	// [gdlr_core_dropcap type="circle" color="#ffffff" background="#212121"]S[/gdlr_core_dropcap]
	add_shortcode('gdlr_core_dropcap', 'gdlr_core_dropcap_shortcode');
	if( !function_exists('gdlr_core_dropcap_shortcode') ){
		function gdlr_core_dropcap_shortcode($atts, $content = ''){
			$atts = shortcode_atts(array(
				'type' => 'circle', // normal, circle, rectangle
				'color' => '#ffffff',
				'background' => '#212121',
			), $atts, 'gdlr_core_dropcap');

			if( $atts['type'] == 'normal' ){
				$atts['background'] = '';
			}

			$extra_class = ' gdlr-core-type-' . $atts['type'];
			return '<span class="gdlr-core-dropcap ' . esc_attr($extra_class) . '" ' . gdlr_core_esc_style(array(
				'color' => $atts['color'],
				'background' => $atts['background']
			)) . ' >' . gdlr_core_escape_content($content) . '</span>';
		}
	}