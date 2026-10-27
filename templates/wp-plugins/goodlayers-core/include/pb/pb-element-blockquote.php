<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('blockquote', 'gdlr_core_pb_element_blockquote'); 
	
	if( !class_exists('gdlr_core_pb_element_blockquote') ){
		class gdlr_core_pb_element_blockquote{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'fa-quote-left',
					'title' => esc_html__('Blockquote', 'goodlayers-core')
				);
			}
			
			// return the element options
			static function get_options(){
				global $gdlr_core_item_pdb;
				
				return array(
					'general' => array(
						'title' => esc_html__('General', 'goodlayers-core'),
						'options' => array(
							'content' => array(
								'title' => esc_html__('Content', 'goodlayers-core'),
								'type' => 'tinymce',
								'default' => esc_html__('Blockquote item sample content', 'goodlayers-core'),
								'wrapper-class' => 'gdlr-core-fullsize'
							),	
							'author' => array(
								'title' => esc_html__('By ( Author )', 'goodlayers-core'),
								'type' => 'text',
								'default' => esc_html__('Blockquote Author', 'goodlayers-core'),
							),	
							'author-position' => array(
								'title' => esc_html__('Author Position', 'goodlayers-core'),
								'type' => 'text',
							),							
						)
					),
					'style' => array(
						'title' => esc_html__('Style', 'goodlayers-core'),
						'options' => array(
							'text-align' => array(
								'title' => esc_html__('Text Align', 'goodlayers-core'),
								'type' => 'radioimage',
								'options' => 'text-align',
								'default' => 'center'
							),
							'size' => array(
								'title' => esc_html__('Size', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'small' => esc_html__('Small', 'goodlayers-core'),
									'medium' => esc_html__('Medium', 'goodlayers-core'),
									'large' => esc_html__('Large', 'goodlayers-core'),
								),
								'default' => 'medium'
							),
						)
					),
					'typography' => array(
						'title' => esc_html__('Typography', 'goodlayers-core'),
						'options' => array(
							'quote-size' => array(
								'title' => esc_html__('Quote Size', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel'
							),
							'quote-font-weight' => array(
								'title' => esc_html__('Quote Font Weight', 'goodlayers-core'),
								'type' => 'text',
							),
							'quote-height' => array(
								'title' => esc_html__('Quote Height', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel'
							),
							'content-size' => array(
								'title' => esc_html__('Content Size', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel'
							),
							'content-font-weight' => array(
								'title' => esc_html__('Content Font Weight', 'goodlayers-core'),
								'type' => 'text',
								'description' => esc_html__('Eg. lighter, bold, normal, 300, 400, 600, 700, 800', 'goodlayers-core')
							),
							'content-font-style' => array(
								'title' => esc_html__('Content Font Style', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'' => esc_html__('Default', 'goodlayers-core'),
									'normal' => esc_html__('Normal', 'goodlayers-core'),
									'italic' => esc_html__('Italic', 'goodlayers-core'),
								),
							),
							'content-letter-spacing' => array(
								'title' => esc_html__('Content Letter Spacing', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
							),
							'content-text-transform' => array(
								'title' => esc_html__('Content Text Transform', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'' => esc_html__('None', 'goodlayers-core'),
									'uppercase' => esc_html__('Uppercase', 'goodlayers-core'),
									'lowercase' => esc_html__('Lowercase', 'goodlayers-core'),
									'capitalize' => esc_html__('Capitalize', 'goodlayers-core'),
								)
							),
							'author-size' => array(
								'title' => esc_html__('Author Size', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel'
							),
							'author-font-weight' => array(
								'title' => esc_html__('Author Font Weight', 'goodlayers-core'),
								'type' => 'text',
								'description' => esc_html__('Eg. lighter, bold, normal, 300, 400, 600, 700, 800', 'goodlayers-core')
							),
							'author-font-style' => array(
								'title' => esc_html__('Content Font Style', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'' => esc_html__('Default', 'goodlayers-core'),
									'normal' => esc_html__('Normal', 'goodlayers-core'),
									'italic' => esc_html__('Italic', 'goodlayers-core'),
								),
							),
							'position-size' => array(
								'title' => esc_html__('Position Size', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel'
							),
							'position-font-weight' => array(
								'title' => esc_html__('Position Font Weight', 'goodlayers-core'),
								'type' => 'text',
								'description' => esc_html__('Eg. lighter, bold, normal, 300, 400, 600, 700, 800', 'goodlayers-core')
							),
						)
					),
					'spacing' => array(
						'title' => esc_html__('Spacing', 'goodlayers-core'),
						'options' => array(
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
				return $content;
			}			
			
			// get the content from settings
			static function get_content( $settings = array() ){
				global $gdlr_core_item_pdb;
				
				// default variable
				if( empty($settings) ){
					$settings = array(
						'content' => esc_html__('Blockquote item sample content', 'goodlayers-core'),
						'author' => esc_html__('Blockquote Author', 'goodlayers-core'),
						'text-align' => 'center', 'size' => 'medium',
						'padding-bottom' => $gdlr_core_item_pdb
					);
				}
				
				// start printing item
				$extra_class  = ' gdlr-core-' . (empty($settings['text-align'])? 'left': $settings['text-align']) . '-align';
				$extra_class .= ' gdlr-core-' . (empty($settings['size'])? 'left': $settings['size']) . '-size';
				$extra_class .= empty($settings['class'])? '': ' ' . $settings['class'];
				$ret  = '<div class="gdlr-core-blockquote-item gdlr-core-item-pdlr gdlr-core-item-pdb ' . esc_attr($extra_class) . '" ';
				if( !empty($settings['padding-bottom']) && $settings['padding-bottom'] != $gdlr_core_item_pdb ){
					$ret .= gdlr_core_esc_style(array('padding-bottom'=>$settings['padding-bottom']));
				}
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' >';
				
				$ret .= '<div class="gdlr-core-blockquote gdlr-core-info-font" >';
				$ret .= '<div class="gdlr-core-blockquote-item-quote gdlr-core-quote-font gdlr-core-skin-icon" ' . gdlr_core_esc_style(array(
					'height' => empty($settings['quote-height'])? '': $settings['quote-height'],
					'font-size' => empty($settings['quote-size'])? '': $settings['quote-size'],
					'font-weight' => empty($settings['quote-font-weight'])? '': $settings['quote-font-weight'],
				)) . ' >' . (($settings['text-align'] == 'right')? '&#8221;': '&#8220;') . '</div>';
				$ret .= '<div class="gdlr-core-blockquote-item-content-wrap" >';
				if( !empty($settings['content']) ){
					$ret .= '<div class="gdlr-core-blockquote-item-content gdlr-core-skin-content" ' . gdlr_core_esc_style(array(
						'font-size' => empty($settings['content-size'])? '': $settings['content-size'],
						'font-weight' => empty($settings['content-font-weight'])? '': $settings['content-font-weight'],
						'font-style' => empty($settings['content-font-style'])? '': $settings['content-font-style'],
						'letter-spacing' => empty($settings['content-letter-spacing'])? '': $settings['content-letter-spacing'],
						'text-transform' => empty($settings['content-text-transform'])? '': $settings['content-text-transform']
					)) . ' >' . gdlr_core_content_filter($settings['content']) . '</div>';
				}
				if( !empty($settings['author']) ){
					$ret .= '<div class="gdlr-core-blockquote-item-author gdlr-core-skin-caption" ' . gdlr_core_esc_style(array(
						'font-size' => empty($settings['author-size'])? '': $settings['author-size'],
						'font-weight' => empty($settings['author-font-weight'])? '': $settings['author-font-weight'],
						'font-style' => empty($settings['author-font-style'])? '': $settings['author-font-style'],
					)) . ' >';
					$ret .= '<span class="gdlr-core-blockquote-item-author-name" >' . gdlr_core_text_filter($settings['author']) . '</span>';
					if( !empty($settings['author-position']) ){
						$ret .= '<span class="gdlr-core-blockquote-item-author-position" ' . gdlr_core_esc_style(array(
							'font-size' => empty($settings['position-size'])? '': $settings['position-size'],
							'font-weight' => empty($settings['position-font-weight'])? '': $settings['position-font-weight'],
					)) . ' >' . gdlr_core_text_filter($settings['author-position']) . '</span>';
					}
					$ret .= '</div>';
				}
				$ret .= '</div>';
				$ret .= '</div>';
				$ret .= '</div>';
				
				return $ret;
			}
			
		} // gdlr_core_pb_element_content
	} // class_exists	