<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('countdown', 'gdlr_core_pb_element_countdown'); 
	
	if( !class_exists('gdlr_core_pb_element_countdown') ){
		class gdlr_core_pb_element_countdown{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'fa-clock-o',
					'title' => esc_html__('Countdown', 'goodlayers-core')
				);
			}
			
			// return the element options
			static function get_options(){
				global $gdlr_core_item_pdb;
				
				return array(
					'general' => array(
						'title' => esc_html__('General', 'goodlayers-core'),
						'options' => array(

							'date' => array(
								'title' => esc_html__('Select Date', 'goodlayers-core'),
								'type' => 'datepicker',
								'default' => date('Y') . '-12-31'
							),
							'time' => array(
								'title' => esc_html__('Time (HH:MM)', 'goodlayers-core'),
								'type' => 'text',
							),
							'alignment' => array(
								'title' => esc_html__('Alignment', 'goodlayers-core'),
								'type' => 'radioimage',
								'options' => 'text-align',
								'default' => 'center'
							),
						)
					),
					'typography' => array(
						'title' => esc_html__('Typography', 'goodlayers-core'),
						'options' => array(
							'number-font-size' => array(
								'title' => esc_html__('Number Font Size', 'goodlayers-core'),
								'type' => 'fontslider',
								'default' => '85px'
							),
							'number-font-weight' => array(
								'title' => esc_html__('Number Font Weight', 'goodlayers-core'),
								'type' => 'text',
								'default' => '300',
								'description' => esc_html__('Eg. lighter, bold, normal, 300, 400, 600, 700, 800', 'goodlayers-core')
							),
							'text-font-size' => array(
								'title' => esc_html__('Text Font Size', 'goodlayers-core'),
								'type' => 'fontslider',
								'default' => '18px'
							),
							'text-font-weight' => array(
								'title' => esc_html__('Text Font Weight', 'goodlayers-core'),
								'type' => 'text',
								'description' => esc_html__('Eg. lighter, bold, normal, 300, 400, 600, 700, 800', 'goodlayers-core')
							),
						)
					),
					'color' => array(
						'title' => esc_html__('Color', 'goodlayers-core'),
						'options' => array(
							'number-font-color' => array(
								'title' => esc_html__('Number Font Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'text-font-color' => array(
								'title' => esc_html__('Text Font Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
						)
					),
					'spacing' => array(
						'title' => esc_html__('Spacing', 'goodlayers-core'),
						'options' => array(
							'space-between-counter' => array(
								'title' => esc_html__('Space Between Counter Number ( left & right )', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'default' => '30px'
							),							
							'space-between-text' => array(
								'title' => esc_html__('Counter Text Top Margin', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'default' => '10px'
							),
							'padding-bottom' => array(
								'title' => esc_html__('Padding Bottom', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'default' => $gdlr_core_item_pdb
							),
						)
					),
				);
			}
			
			// get the preview for page builder
			static function get_preview( $settings = array() ){
				$content  = self::get_content($settings);
				$id = mt_rand(0, 9999);
				
				ob_start();
?><script id="gdlr-core-preview-countdown-<?php echo esc_attr($id); ?>" >
jQuery(document).ready(function(){
	jQuery('#gdlr-core-preview-countdown-<?php echo esc_attr($id); ?>').parent().gdlr_core_countdown_item();
});
</script><?php	
				$content .= ob_get_contents();
				ob_end_clean();		

				return $content;		
			}			
			
			// get the content from settings
			static function get_content( $settings = array() ){
				global $gdlr_core_item_pdb;
				
				// default variable
				if( empty($settings) ){
					$settings = array(
						'date' => date('Y') . '-12-31',
						'padding-bottom' => $gdlr_core_item_pdb
					);
				}
				
				// start printing item
				$extra_class  = empty($settings['class'])? '': $settings['class'];
				$ret  = '<div class="gdlr-core-countdown-item gdlr-core-item-pdlr gdlr-core-item-pdb ' . esc_attr($extra_class) . '" ';
				if( !empty($settings['padding-bottom']) && $settings['padding-bottom'] != $gdlr_core_item_pdb ){
					$ret .= gdlr_core_esc_style(array('padding-bottom'=>$settings['padding-bottom']));
				}
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' >';
				if( !empty($settings['date']) ){
					$due_date = strtotime($settings['date'] . (empty($settings['time'])? '': ' ' . $settings['time']));	
					$current_date = strtotime(current_time('mysql'));

					if( $due_date > $current_date ){
						$total_time = $due_date - $current_date;
						$day = intval($total_time / 86400);
						
						$total_time = $total_time % 86400;
						$hrs = intval($total_time / 3600);
						
						$total_time = $total_time % 3600;
						$min = intval($total_time / 60);
						$sec = $total_time % 60;
							
						$countdown_wrap_class = ' gdlr-core-' . (empty($settings['alignment'])? 'center': $settings['alignment']) . '-align';
						$block_attr = array(
							'margin-left' => (empty($settings['space-between-counter']) || $settings['space-between-counter'] == '30px')? '': $settings['space-between-counter'],
							'margin-right' => (empty($settings['space-between-counter']) || $settings['space-between-counter'] == '30px')? '': $settings['space-between-counter'],
						);
						$time_attr = array(
							'font-size' => (empty($settings['number-font-size']) || $settings['number-font-size'] == '85px')? '': $settings['number-font-size'],
							'font-weight' => (empty($settings['number-font-weight']) || $settings['number-font-weight'] == '300')? '': $settings['number-font-weight'],
							'color' => empty($settings['number-font-color'])? '': $settings['number-font-color'],
						);
						$text_attr = array(
							'font-size' => (empty($settings['text-font-size']) || $settings['text-font-size'] == '18px')? '': $settings['text-font-size'],
							'font-weight' => empty($settings['text-font-weight'])? '': $settings['text-font-weight'],
							'color' => empty($settings['text-font-color'])? '': $settings['text-font-color'],
							'margin-top' => empty($settings['space-between-text'])? '': $settings['space-between-text'],
						);
						$ret .= '<div class="gdlr-core-countdown-wrap gdlr-core-js ' . esc_attr($countdown_wrap_class) . '" >';
						$ret .= '<div class="gdlr-core-countdown-block gdlr-core-block-day" ' . gdlr_core_esc_style($block_attr) . ' >';
						$ret .= '<span class="gdlr-core-time gdlr-core-day" ' . gdlr_core_esc_style($time_attr) . ' >' . $day . '</span>';
						$ret .= '<span class="gdlr-core-unit gdlr-core-skin-caption" ' . gdlr_core_esc_style($text_attr) . ' >' . esc_html__('Days', 'goodlayers-core') . '</span>';
						$ret .= '</div>';
						
						$ret .= '<div class="gdlr-core-countdown-block gdlr-core-block-hrs" ' . gdlr_core_esc_style($block_attr) . ' >';
						$ret .= '<span class="gdlr-core-time gdlr-core-hrs"  ' . gdlr_core_esc_style($time_attr) . ' >' . $hrs . '</span>';
						$ret .= '<span class="gdlr-core-unit gdlr-core-skin-caption" ' . gdlr_core_esc_style($text_attr) . ' >' . esc_html__('Hours', 'goodlayers-core') . '</span>';
						$ret .= '</div>';

						$ret .= '<div class="gdlr-core-countdown-block gdlr-core-block-min" ' . gdlr_core_esc_style($block_attr) . ' >';
						$ret .= '<span class="gdlr-core-time gdlr-core-min" ' . gdlr_core_esc_style($time_attr) . ' >' . $min . '</span>';
						$ret .= '<span class="gdlr-core-unit gdlr-core-skin-caption" ' . gdlr_core_esc_style($text_attr) . ' >' . esc_html__('Mins', 'goodlayers-core') . '</span>';
						$ret .= '</div>';

						$ret .= '<div class="gdlr-core-countdown-block gdlr-core-block-sec" ' . gdlr_core_esc_style($block_attr) . ' >';
						$ret .= '<span class="gdlr-core-time gdlr-core-sec" ' . gdlr_core_esc_style($time_attr) . ' >' . $sec . '</span>';
						$ret .= '<span class="gdlr-core-unit gdlr-core-skin-caption" ' . gdlr_core_esc_style($text_attr) . ' >' . esc_html__('Secs', 'goodlayers-core') . '</span>';
						$ret .= '</div>';
						$ret .= '<div class="clear"></div>';
			 			$ret .= '</div>';	
					}
				}
				$ret .= '</div>';
				
				return $ret;
			}
			
		} // gdlr_core_pb_element_countdown
	} // class_exists	