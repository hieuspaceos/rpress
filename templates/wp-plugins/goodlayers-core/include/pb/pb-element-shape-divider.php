<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('shape-divider', 'gdlr_core_pb_element_shape_divider'); 
	
	if( !class_exists('gdlr_core_pb_element_shape_divider') ){
		class gdlr_core_pb_element_shape_divider{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'fa-align-justify',
					'title' => esc_html__('Shape Divider', 'goodlayers-core')
				);
			}
			
			// return the element options
			static function get_options(){
				global $gdlr_core_item_pdb;
				
				return array(
					'general' => array(
						'title' => esc_html__('General', 'goodlayers-core'),
						'options' => array(
							'position' => array(
								'title' => esc_html__('Position', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'top' => esc_html__('Top', 'goodlayers-core'),
									'bottom' => esc_html__('Bottom', 'goodlayers-core')
								)
							),
							'shape' => array(
								'title' => esc_html__('Shape', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'custom' => esc_html__('Custom SVG', 'goodlayers-core'),
									'custom-image' => esc_html__('Custom Image', 'goodlayers-core'),
									'book' => esc_html__('Book', 'goodlayers-core'),
									'curve' => esc_html__('Curve', 'goodlayers-core'),
									'curve-asymmetrical' => esc_html__('Curve Asymmetrical', 'goodlayers-core'),
									'fan-opacity' => esc_html__('Fan Opacity', 'goodlayers-core'),
									'mountains' => esc_html__('Mountains', 'goodlayers-core'),
									'pyramids' => esc_html__('Pyramids', 'goodlayers-core'),
									'tilt' => esc_html__('Tilt', 'goodlayers-core'),
									'tilt-opacity' => esc_html__('Tilt Opacity', 'goodlayers-core'),
									'triangle' => esc_html__('Triangle', 'goodlayers-core'),
									'triangle-asymmetrical' => esc_html__('Triangle Asymmetrical', 'goodlayers-core'),
									'waves' => esc_html__('Waves', 'goodlayers-core'),
									'waves-pattern' => esc_html__('Waves Pattern', 'goodlayers-core'),
									'zig-zag' => esc_html__('Zig Zag', 'goodlayers-core'),
								),
								'default' => 'book',
							),
							'custom-shape' => array(
								'title' => esc_html__('Custom Shape', 'goodlayers-core'),
								'type' => 'upload',
								'condition' => array( 'shape' => 'custom' )
							),
							'custom-image' => array(
								'title' => esc_html__('Custom Shape', 'goodlayers-core'),
								'type' => 'upload',
								'condition' => array( 'shape' => 'custom-image' )
							),
							'custom-image-height' => array(
								'title' => esc_html__('Custom Image Height', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'condition' => array( 'shape' => 'custom-image' )
							),
							'inverted' => array(
								'title' => esc_html__('Inverted', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'disable',
								'condition' => array( 'shape' => array('book', 'curve', 'curve-asymmetrical', 'pyramids', 'triangle', 'triangle-asymmetrical', 'waves', 'custom-image') )
							),
							'flip' => array(
								'title' => esc_html__('Flip Horizontal', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'disable',
								'condition' => array( 'shape' => array('custom', 'book', 'curve', 'curve-asymmetrical', 'fan-opacity', 'mountains', 'pyramids', 'tilt', 'tilt-opacity', 'triangle', 'triangle-asymmetrical', 'waves', 'waves-pattern', 'zig-zag') )
							),
							'vflip' => array(
								'title' => esc_html__('Flip Vertical', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'disable',
								'condition' => array('shape' => 'custom')
							),
							'opacity' => array(
								'title' => esc_html__('Opacity', 'goodlayers-core'),
								'type' => 'text',
								'description' => esc_html__('Fill the number between 0.01 to 1', 'goodlayers-core'),
								'condition' => array( 'shape' => array('custom', 'book', 'curve', 'curve-asymmetrical', 'fan-opacity', 'mountains', 'pyramids', 'tilt', 'tilt-opacity', 'triangle', 'triangle-asymmetrical', 'waves', 'waves-pattern', 'zig-zag') )
							), 
							'color' => array(
								'title' => esc_html__('Color', 'goodlayers-core'),
								'type' => 'colorpicker',
								'condition' => array( 'shape' => array('custom', 'book', 'curve', 'curve-asymmetrical', 'fan-opacity', 'mountains', 'pyramids', 'tilt', 'tilt-opacity', 'triangle', 'triangle-asymmetrical', 'waves', 'waves-pattern', 'zig-zag') )
							),
							'width' => array(
								'title' => esc_html__('Width % ( Min value is 100 )', 'goodlayers-core'),
								'type' => 'text',
								'condition' => array( 'shape' => array('custom', 'book', 'curve', 'curve-asymmetrical', 'fan-opacity', 'mountains', 'pyramids', 'tilt', 'tilt-opacity', 'triangle', 'triangle-asymmetrical', 'waves', 'waves-pattern', 'zig-zag') )
							),
							'height' => array(
								'title' => esc_html__('Height', 'goodlayers-core'),
								'data-input-type' => 'pixel',
								'type' => 'text',
								'condition' => array( 'shape' => array('custom', 'book', 'curve', 'curve-asymmetrical', 'fan-opacity', 'mountains', 'pyramids', 'tilt', 'tilt-opacity', 'triangle', 'triangle-asymmetrical', 'waves', 'waves-pattern', 'zig-zag') )
							),
							'hide-this-item-in' => array(
								'title' => esc_html__('Hide This Item In', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'none' => esc_html__('None', 'goodlayers-core'),
									'desktop' => esc_html__('Desktop', 'goodlayers-core'),
									'desktop-tablet' => esc_html__('Desktop & Tablet', 'goodlayers-core'),
									'tablet' => esc_html__('Tablet', 'goodlayers-core'),
									'tablet-mobile' => esc_html__('Tablet & Mobile', 'goodlayers-core'),
									'mobile' => esc_html__('Mobile', 'goodlayers-core'),
								)
							), 
							'z-index' => array(
								'title' => esc_html__('Z Index', 'goodlayers-core'),
								'type' => 'text'
							)
						)
					),
				);
			}
			
			// get the preview for page builder
			static function get_preview( $settings = array() ){
				$content  = self::get_content($settings, true);
				$id = mt_rand(0, 9999);
				
				ob_start();
?><script id="gdlr-core-preview-shape-divider-<?php echo esc_attr($id); ?>" >
jQuery(document).ready(function(){
	jQuery('#gdlr-core-preview-shape-divider-<?php echo esc_attr($id); ?>').parent();
});
</script><?php	
				$content .= ob_get_contents();
				ob_end_clean();
				
				return $content;
			}			
			
			// get the content from settings
			static function get_content( $settings = array(), $preview = false ){
				// default variable
				if( empty($settings) ){
					$settings = array();
				}

				$settings['shape'] = empty($settings['shape'])? 'book': $settings['shape'];
				$settings['position'] = empty($settings['position'])? 'top': $settings['position'];
				$settings['inverted'] = empty($settings['inverted'])? 'disable': $settings['inverted'];
				$settings['flip'] = empty($settings['flip'])? 'disable': $settings['flip'];
				$settings['vflip'] = empty($settings['vflip'])? 'disable': $settings['vflip'];

				$custom_css = '';
				if( !empty($settings['color']) ){
					$custom_css .= '#id svg path{ fill: ' . $settings['color'] . '; }';
				}
				if( !empty($settings['opacity']) ){
					$custom_css .= '#id svg path{ opacity: ' . $settings['opacity'] . '; }';
				}
				if( !empty($settings['width']) ){
					$width = intval($settings['width']);
					if( $width > 100 ){
						$custom_css .= '#id svg{ width: ' . $width . '%; }';
					}
				}
				if( !empty($settings['height']) ){
					$custom_css .= '#id svg{ height: ' . $settings['height'] . '; }';
				}

				if( !empty($custom_css) && empty($settings['id']) ){
					global $gdlr_core_shape_divider_id; 

					if( $preview ){
						$gdlr_core_shape_divider_id = empty($gdlr_core_shape_divider_id)? array(): $gdlr_core_shape_divider_id;
						
						// generate unique id so it does not get overwritten in admin area
						$rnd_id = mt_rand(0, 99999);
						while( in_array($rnd_id, $gdlr_core_shape_divider_id) ){
							$rnd_id = mt_rand(0, 99999);
						}
						$gdlr_core_shape_divider_id[] = $rnd_id;
						$settings['id'] = 'gdlr-core-shape-divider-' . $rnd_id;
					}else{
						$gdlr_core_shape_divider_id = empty($gdlr_core_shape_divider_id)? 1: $gdlr_core_shape_divider_id + 1;
						$settings['id'] = 'gdlr-core-shape-divider-' . $gdlr_core_shape_divider_id;
					}

					$custom_css = str_replace('#id', '#' . $settings['id'], $custom_css);
				}

				$additional_class  = ' gdlr-core-pos-' . $settings['position'];
				$additional_class .= ($preview)? ' gdlr-core-preview': '';
				$additional_class .= ($settings['flip'] == 'enable')? ' gdlr-core-flip': '';
				if( !empty($settings['hide-this-item-in']) && $settings['hide-this-item-in'] != 'none' ){
					$additional_class .= ' gdlr-core-hide-in-' . $settings['hide-this-item-in'];
				}

				if( $settings['shape'] == 'custom' ){
					if( $settings['position'] == 'top' && $settings['vflip'] == 'enable' ){
						$additional_class .= ' gdlr-core-inverted';
					}else if( $settings['position'] == 'bottom' && $settings['vflip'] == 'disable' ){
						$additional_class .= ' gdlr-core-inverted';
					}
				}else if( in_array($settings['shape'], array('book', 'curve', 'curve-asymmetrical', 'pyramids', 'triangle', 'triangle-asymmetrical', 'waves')) ){
					if( $settings['inverted'] == 'enable' ){
						$settings['shape'] .= '-negative';
						$additional_class .= ' gdlr-core-inverted';
					}
				}else if( $settings['shape'] == 'custom-image' ){
					if( $settings['inverted'] == 'enable' ){	
						$additional_class .= ' gdlr-core-inverted';
					}
				} 	

				if( $preview ){
					$custom_css = '<style>' . $custom_css . '</style>';
				}else{
					gdlr_core_add_inline_style($custom_css);
					$custom_css = '';
				}

				// start printing item
				$ret = '';
				$ret .= '<div class="gdlr-core-shape-divider-item" ';
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' >';

				if( $preview && $settings['position'] == 'top' ){
					$ret .= '<div class="gdlr-core-preview-text" >';
					$ret .= esc_html__('This item will shows at the very top of the section on front end of the site.', 'goodlayers-core');
					$ret .= '</div>';
				}

				$css_atts = array(
					'z-index' => empty($settings['z-index'])? '': $settings['z-index']
				);
				if( $settings['shape'] == 'custom-image' && !empty($settings['custom-image']) && !empty($settings['custom-image-height']) ){
					$css_atts['background-image'] = $settings['custom-image'];
					$css_atts['height'] = $settings['custom-image-height'];

					$additional_class .= ' gdlr-core-custom-image-with-height ';
				}

				$ret .= '<div class="gdlr-core-shape-divider-wrap ' . esc_attr($additional_class) . '" ' . gdlr_core_esc_style($css_atts) . ' >';
				if( $settings['shape'] == 'custom-image' && !empty($settings['custom-image']) ){
					if( empty($settings['custom-image-height']) ){
						$ret .= gdlr_core_get_image($settings['custom-image']);
					}
				}else if( $settings['shape'] == 'custom' && !empty($settings['custom-shape']) ){
					$ret .= self::get_shape_content(get_attached_file($settings['custom-shape']), true);
				}else{
					$ret .= self::get_shape_content($settings['shape']);
				}
				$ret .= '</div>';

				if( $preview && $settings['position'] == 'bottom' ){
					$ret .= '<div class="gdlr-core-preview-text" >';
					$ret .= esc_html__('This item will shows at the very bottom of the section on front end of the site.', 'goodlayers-core');
					$ret .= '</div>';
				}

				$ret .= '</div>';
				$ret .= $custom_css;
				
				return $ret;
			}

			static function get_shape_content($shape, $file = false){

				if( $file ){
					if( !empty($shape) ){
						$ret = file_get_contents($shape);
					}else{
						return '';
					}
				}else{
					$ret = file_get_contents(GDLR_CORE_LOCAL . '/include/css/shapes/' . $shape . '.svg');
				}

				// remove specific attribute out
				$remove_att_list = array(
					array('id="', '"'),
					array('<?xml', '?>'),
				);
				foreach($remove_att_list as $remove_att){
					$start = $remove_att[0];
					$end = $remove_att[1];

					$start_pos = strpos($ret, $start, 0);
					while( $start_pos !== false ){
						$end_pos = strpos($ret, $end, $start_pos + strlen($start));
						if( $end_pos === false ) return;

						$ret_temp = substr($ret, 0, $start_pos) . substr($ret, $end_pos + strlen($end));
						$ret = $ret_temp;

						$start_pos = strpos($ret, $start, 0);
					}
				}

				return $ret;
			}
			
		} // gdlr_core_pb_element_shape_divider
	} // class_exists	