<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('timeline', 'gdlr_core_pb_element_timeline'); 
	
	if( !class_exists('gdlr_core_pb_element_timeline') ){
		class gdlr_core_pb_element_timeline{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'fa-folder-o',
					'title' => esc_html__('Timeline', 'goodlayers-core')
				);
			}
			
			// return the element options
			static function get_options(){
				global $gdlr_core_item_pdb;
				
				return array(
					'general' => array(
						'title' => esc_html__('General', 'goodlayers-core'),
						'options' => array(
							'tabs' => array(
								'title' => esc_html__('Add New Tab', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'tabs',
								'wrapper-class' => 'gdlr-core-fullsize',
								'options' => array(
									'date' => array(
										'title' => esc_html__('Date', 'goodlayers-core'),
										'type' => 'text'
									),
									'title' => array(
										'title' => esc_html__('Title', 'goodlayers-core'),
										'type' => 'text'
									),
									'caption' => array(
										'title' => esc_html__('Caption', 'goodlayers-core'),
										'type' => 'text'
									),
									'content' => array(
										'title' => esc_html__('Content', 'goodlayers-core'),
										'type' => 'tmce'
									),
								),
								'default' => array(
									array(
										'date' => esc_html__('1985-1990', 'goodlayers-core'),
										'title' => esc_html__('Timeline', 'goodlayers-core'),
										'caption' => esc_html__('Caption Text', 'goodlayers-core'),
										'content' => esc_html__('Content Area', 'goodlayers-core'),
									),
									array(
										'date' => esc_html__('1985-1990', 'goodlayers-core'),
										'title' => esc_html__('Timeline', 'goodlayers-core'),
										'caption' => esc_html__('Caption Text', 'goodlayers-core'),
										'content' => esc_html__('Content Area', 'goodlayers-core'),
									),
								)
							),
						),
					),
					'spacing' => array(
						'title' => esc_html__('Spacing', 'goodlayers-core'),
						'options' => array(
							'date-width' => array(
								'title' => esc_html__('Timeline Date Width', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'default' => '300px',
							),
							'date-right-padding' => array(
								'title' => esc_html__('Timeline Date Right Padding', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'description' => esc_html__('Leave this field blank for default value.', 'goodlayers-core')
							),
							'padding-bottom' => array(
								'title' => esc_html__('Padding Bottom ( Item )', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'default' => $gdlr_core_item_pdb
							)
						)
					)
				);
			}

			// get the preview for page builder
			static function get_preview( $settings = array() ){
				$content  = self::get_content($settings, true);
				return $content;
			}		
			
			// get the content from settings
			static function get_content( $settings = array(), $preview = false ){
				global $gdlr_core_item_pdb;

				// default variable
				if( empty($settings) ){
					$settings = array(
						'tabs' => array(
							array(
								'date' => esc_html__('1985-1990', 'goodlayers-core'),
								'title' => esc_html__('Timeline', 'goodlayers-core'),
								'caption' => esc_html__('Caption Text', 'goodlayers-core'),
								'content' => esc_html__('Content Area', 'goodlayers-core'),
							),
							array(
								'date' => esc_html__('1985-1990', 'goodlayers-core'),
								'title' => esc_html__('Timeline', 'goodlayers-core'),
								'caption' => esc_html__('Caption Text', 'goodlayers-core'),
								'content' => esc_html__('Content Area', 'goodlayers-core'),
							),
						),
					);
				}

				$date_width = (empty($settings['date-width']) || $settings['date-width'] == '300px')? '': $settings['date-width']; 
				$date_right_padding = empty($settings['date-right-padding'])? '': $settings['date-right-padding']; 
				
				// start printing item
				$ret  = '<div class="gdlr-core-timeline-item gdlr-core-item-pdb gdlr-core-item-pdlr" ';
				if( !empty($settings['padding-bottom']) && $settings['padding-bottom'] != $gdlr_core_item_pdb ){
					$ret .= gdlr_core_esc_style(array('padding-bottom'=>$settings['padding-bottom']));
				}
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' >';

				if( !empty($settings['tabs']) ){
					foreach( $settings['tabs'] as $tab ){
						$ret .= '<div class="gdlr-core-timeline-item-list clearfix" >';
						$ret .= '<div class="gdlr-core-timeline-item-date" ' . gdlr_core_esc_style(array(
							'width' => $date_width,
							'padding-right' => $date_right_padding,
						)) . ' >';
						$ret .= gdlr_core_text_filter($tab['date']);
						$ret .= '<div class="gdlr-core-timeline-item-bullet" >';
						$ret .= '<div class="gdlr-core-timeline-item-divider" ></div>';
						$ret .= '</div>'; // gdlr-core-timeline-item-bullet
						$ret .= '</div>'; // gdlr-core-timeline-item-date
						
						$ret .= '<div class="gdlr-core-timeline-item-content-wrap" >';
						$ret .= '<div class="gdlr-core-timeline-item-title" >' . gdlr_core_text_filter($tab['title']) . '</div>';
						$ret .= '<div class="gdlr-core-timeline-item-caption" >' . gdlr_core_text_filter($tab['caption']) . '</div>';
						$ret .= '<div class="gdlr-core-timeline-item-content" >' . gdlr_core_content_filter($tab['content']) . '</div>';
						$ret .= '</div>'; // gdlr-core-timeline-content-wrap
						$ret .= '</div>'; // gdlr-core-timeline-item-list
					}
				}

				$ret .= '</div>'; // gdlr-core-timeline-item

				return $ret;
			}			
			
		} // gdlr_core_pb_element_timeline
	} // class_exists	