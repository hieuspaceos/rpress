<?php
	/*	
	*	Goodlayers Item
	*/
	
	// [gdlr_core_dropdown_tab]
	// [gdlr_core_tab title="" ]
	// [/gdlr_core_dropdown_tab]
	add_shortcode('gdlr_core_dropdown_tab', 'gdlr_core_dropdown_tab_shortcode');
	if( !function_exists('gdlr_core_dropdown_tab_shortcode') ){
		function gdlr_core_dropdown_tab_shortcode($atts, $content = ''){
			$atts = shortcode_atts(array(), $atts, 'gdlr_core_dropdown_tab');
			
			$ret  = apply_filters('gdlr_core_dropdown_tab_content', $atts, $content);
			if( !empty($ret) ){ return $ret; }

			global $gdlr_core_tabs;
			$gdlr_core_tabs = array();

			do_shortcode($content);

			$active_tab = 0;

			$ret  = '<div class="gdlr-core-dropdown-tab gdlr-core-js clearfix" >';
			$ret .= '<div class="gdlr-core-dropdown-tab-title" >';
			$ret .= '<span class="gdlr-core-head">' . $gdlr_core_tabs[$active_tab]['title'] . '</span>';
			
			$count = 0;
			$ret .= '<div class="gdlr-core-dropdown-tab-head-wrap" >';
			foreach( $gdlr_core_tabs as $gdlr_core_tab ){
				$ret .= '<div class="gdlr-core-dropdown-tab-head ' . (($count == $active_tab)? 'gdlr-core-active': '') . '" data-index="' . esc_attr($count) . '" >' . $gdlr_core_tab['title'] . '</div>';

				$count ++;
			}
			$ret .= '</div>'; // gdlr-core-dropdown-tab-head-wrap
			$ret .= '</div>'; // gdlr-core-dropdown-tab-title

			$count = 0;
			$ret .= '<div class="gdlr-core-dropdown-tab-content-wrap" >';
			foreach( $gdlr_core_tabs as $gdlr_core_tab ){
				$ret .= '<div class="gdlr-core-dropdown-tab-content ' . (($count == $active_tab)? 'gdlr-core-active': '') . '" data-index="' . esc_attr($count) . '" >' . $gdlr_core_tab['content'] . '</div>';

				$count ++;
			}
			$ret .= '</div>';
			$ret .= '</div>'; // gdlr-core-dropdown-tab

			return $ret;
		}
	}