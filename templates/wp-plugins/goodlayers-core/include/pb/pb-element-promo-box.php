<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('promo-box', 'gdlr_core_pb_element_promo_box'); 
	
	if( !class_exists('gdlr_core_pb_element_promo_box') ){
		class gdlr_core_pb_element_promo_box{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'icon_ribbon_alt',
					'title' => esc_html__('Promo Box', 'goodlayers-core')
				);
			}
			
			// return the element options
			static function get_options(){
				global $gdlr_core_item_pdb;
				
				return array(
					'general' => array(
						'title' => esc_html__('General', 'goodlayers-core'),
						'options' => array(
							'image' => array(
								'title' => esc_html__('Image', 'goodlayers-core'),
								'type' => 'upload',
							),
							'thumbnail-size' => array(
								'title' => esc_html__('Thumbnail Size', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => 'thumbnail-size',
								'default' => 'full'
							),
							'title' => array(
								'title' => esc_html__('Title', 'goodlayers-core'),
								'type' => 'text',
								'default' => esc_html__('Promo Box Item Title', 'goodlayers-core'),
							),
							'content' => array(
								'title' => esc_html__('Content', 'goodlayers-core'),
								'type' => 'tinymce',
								'default' => esc_html__('Promo box item sample content', 'goodlayers-core'),
								'wrapper-class' => 'gdlr-core-fullsize'
							),	
							'text-align' => array(
								'title' => esc_html__('Text Align', 'goodlayers-core'),
								'type' => 'radioimage',
								'options' => 'text-align',
								'default' => 'center'
							),					
						)
					),
					'frame' => array(
						'title' => esc_html__('Frame Style', 'goodlayers-core'),
						'options' => array(
							'enable-frame' => array(
								'title' => esc_html__('Enable Frame', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'enable'
							),  
							'enable-shadow' => array(
								'title' => esc_html__('Enable Shadow', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'enable',
								'condition' => array( 'enable-frame' => 'enable' )
							), 
							'frame-padding' => array(
								'title' => esc_html__('Frame Padding', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'data-input-type' => 'pixel',
								'default' => array( 'top'=>'30px', 'right'=>'30px', 'bottom'=>'10px', 'left'=>'30px', 'settings'=>'unlink' ),
								'condition' => array( 'enable-frame' => 'enable' )
							),
							'frame-background-color' => array(
								'title' => esc_html__('Frame Background Color', 'goodlayers-core'),
								'type' => 'colorpicker',
								'condition' => array( 'enable-frame' => 'enable' )
							),
							'frame-border-width' => array(
								'title' => esc_html__('Frame Border Width', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'data-input-type' => 'pixel',
								'default' => array( 'top'=>'0px', 'right'=>'1px', 'bottom'=>'1px', 'left'=>'1px', 'settings'=>'unlink' ),
								'condition' => array( 'enable-frame' => 'enable' )
							),
							'frame-border-color' => array(
								'title' => esc_html__('Frame Border Color', 'goodlayers-core'),
								'type' => 'colorpicker',
								'condition' => array( 'enable-frame' => 'enable' )
							), 
						)
					),
					'typography' => array(
						'title' => esc_html__('Typography', 'goodlayers-core'),
						'options' => array(
							'title-size' => array(
								'title' => esc_html__('Title Size', 'goodlayers-core'),
								'type' => 'fontslider',
								'default' => '14px'
							),
							'content-size' => array(
								'title' => esc_html__('Content Size', 'goodlayers-core'),
								'type' => 'fontslider',
								'default' => '14px'
							),
						)
					),
					'color' => array(
						'title' => esc_html__('Color', 'goodlayers-core'),
						'options' => array(
							'title-color' => array(
								'title' => esc_html__('Title Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'content-color' => array(
								'title' => esc_html__('Content Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							)
						)
					),
					'spacing' => array(
						'title' => esc_html__('Spacing', 'goodlayers-core'),
						'options' => array(
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
				$content  = self::get_content($settings);
				return $content;
			}			
			
			// get the content from settings
			static function get_content( $settings = array() ){
				global $gdlr_core_item_pdb;
				
				// default variable
				if( empty($settings) ){
					$settings = array(
						'image' => GDLR_CORE_URL . '/include/images/goodlayers.png',
						'title' => esc_html__('Promo Box Item Title', 'goodlayers-core'),
						'content' => esc_html__('Promo box item sample content', 'goodlayers-core'),
						'enable-frame' => 'enable', 'enable-shadow' => 'enable',
						
						'padding-bottom' => $gdlr_core_item_pdb
					);
				}
				
				// default-value
				$settings['title-size'] = (empty($settings['title-size']) || $settings['title-size'] == '14px')? '': $settings['title-size'];
				$settings['content-size'] = (empty($settings['content-size']) || $settings['content-size'] == '14px')? '': $settings['content-size'];

				// start printing item
				$extra_class  = 'gdlr-core-' . (empty($settings['text-align'])? 'left': $settings['text-align']) . '-align';
				$extra_class .= empty($settings['class'])? '': ' ' . $settings['class'];
				$ret  = '<div class="gdlr-core-promo-box-item gdlr-core-item-pdlr gdlr-core-item-pdb ' . esc_attr($extra_class) . '" ';
				if( !empty($settings['padding-bottom']) && $settings['padding-bottom'] != $gdlr_core_item_pdb ){
					$ret .= gdlr_core_esc_style(array('padding-bottom'=>$settings['padding-bottom']));
				}
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' >';

				$frame_atts = array();
				$frame_class = 'gdlr-core-' . (empty($settings['text-align'])? 'left': $settings['text-align']) . '-align';
				if( !empty($settings['enable-frame']) && $settings['enable-frame'] == 'enable' ){
					$frame_class .= ' gdlr-core-with-frame';
					if( !empty($settings['enable-shadow']) && $settings['enable-shadow'] == 'enable' ){
						$frame_class .= ' gdlr-core-with-shadow';
					}
					$frame_atts = array(
						'background-color' => empty($settings['frame-background-color'])? '': $settings['frame-background-color'],
						'border-color' => empty($settings['frame-border-color'])? '': $settings['frame-border-color'],
						'padding' => (empty($settings['frame-padding']) || $settings['frame-padding'] == array(
								'top'=>'30px', 'right'=>'30px', 'bottom'=>'10px', 'left'=>'30px', 'settings'=>'unlink'
							))? '': $settings['frame-padding'],
						'border-width' => (empty($settings['frame-border-width']) || $settings['frame-border-width'] == array(
								'top'=>'0px', 'right'=>'1px', 'bottom'=>'1px', 'left'=>'1px', 'settings'=>'unlink'
							))? '': $settings['frame-border-width']
					);
				}
				
				$ret .= '<div class="gdlr-core-promo-box">';
				if( !empty($settings['image']) ){
					$thumbnail_size = empty($settings['thumbnail-size'])? 'full': $settings['thumbnail-size'];
					$ret .= '<div class="gdlr-core-promo-box-item-image gdlr-core-media-image" >' . gdlr_core_get_image($settings['image'], $thumbnail_size) . '</div>';
				}
				$ret .= '<div class="gdlr-core-promo-box-content-wrap ' . esc_attr($frame_class) . '" ' . gdlr_core_esc_style($frame_atts) . ' >';
				if( !empty($settings['title']) ){
					$ret .= '<h3 class="gdlr-core-promo-box-item-title" ' . gdlr_core_esc_style(array(
						'font-size'=>$settings['title-size'],
						'color'=>empty($settings['title-color'])? '': $settings['title-color']
					)) . ' >' . gdlr_core_text_filter($settings['title']) . '</h3>';
				}
				if( !empty($settings['content']) ){
					$ret .= '<div class="gdlr-core-promo-box-item-content" ' . gdlr_core_esc_style(array(
						'font-size'=>$settings['content-size'],
						'color'=>empty($settings['content-color'])? '': $settings['content-color']
					)) . ' >' . gdlr_core_content_filter($settings['content']) . '</div>';
				}
				$ret .= '</div>'; // gdlr-core-promo-box-content-wrap
				$ret .= '</div>'; // gdlr-core-promo-box
				
				$ret .= '</div>'; // gdlr-core-promo-box-item
				
				return $ret;
			}
			
		} // gdlr_core_pb_element_promo_box
	} // class_exists	